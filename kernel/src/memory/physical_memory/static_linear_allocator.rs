use core::mem::size_of;
use core::ptr;
use bit::BitIndex;
use bitfield::Bit;
use limine::memory_map;
use limine::memory_map::EntryType;
use linked_list_allocator::align_up;
use rlibc::memset;
use bit;
use crate::{HHDM_OFFSET, set_bit, test_bit};
use crate::memory::{PAGE_SIZE, PhysicalAddress};
use crate::memory::physical_memory::{Frame, FrameAllocator};

struct PmmModule {
    start_address: PhysicalAddress,

    bitmap_size: usize,
    bitmap_entry_count: usize,
    last_free: Option<usize>,
    bitmap: *mut u8,

    next: Option<*mut PmmModule>,
}
unsafe impl Send for PmmModule {}
unsafe impl Sync for PmmModule {}
impl PmmModule {
    fn init(start_address: PhysicalAddress, size: usize, memory_maps_start: *mut u8) -> Self {
        let frame_count = size.div_ceil(PAGE_SIZE);

        let module = Self {
            start_address,

            bitmap_size: frame_count.div_ceil(8),
            bitmap_entry_count: frame_count,
            bitmap: memory_maps_start,

            last_free: Some(0),
            next: None,
        };

        unsafe {
            memset(module.bitmap, 0, module.bitmap_size);
        }

        module
    }

    fn allocate_frames(&mut self, count: usize) -> Option<PhysicalAddress> {
        if let Some(last_free) = self.last_free {
            let alloc = self.start_address + last_free * PAGE_SIZE;
            let bit_base = (alloc - self.start_address) / PAGE_SIZE;

            let byte_index = bit_base / 8;
            let bit_index =  7 - (bit_base % 8);
            unsafe { *self.bitmap.add(byte_index) }.set_bit(bit_index, true);

            for i in bit_base..self.bitmap_entry_count {
                let byte_index = i / 8;
                let bit_index = 7 - (i % 8);
                if Bit::bit(&unsafe { *self.bitmap.add(byte_index) }, bit_index) {
                    self.last_free = Some(bit_base + i);
                    break;
                }
            }

            return Some(alloc);
        }

        None
    }
}

pub struct StaticLinearAllocator {
    root_module: &'static mut PmmModule,
}
impl StaticLinearAllocator {
    pub fn new(memory_regions: &[&memory_map::Entry]) -> Result<Self, &'static str> {
        // Calculate how much memory will be necessary to accommodate the allocator
        let buffer_size = memory_regions
            .iter()
            .filter(|entry| entry.entry_type == EntryType::USABLE)
            .fold(0, |acc, entry|
                acc + size_of::<PmmModule>() * 2 + entry.length.div_ceil(PAGE_SIZE as u64).div_ceil(8) as usize);

        serial_println!("pmm: allocator requires {} bytes", buffer_size);

        // Find an available region large enough to fit everything
        let containing_entry = memory_regions
            .iter()
            .enumerate()
            .find(|entry| entry.1.entry_type == EntryType::USABLE && entry.1.length >= buffer_size as u64)
            .ok_or("pmm: could not find a suitable memory region to hold the pmm")?;
        let buffer_start = align_up(containing_entry.1.base as usize + *HHDM_OFFSET, PAGE_SIZE);
        let mut meta_buffer = buffer_start as *mut u8;

        serial_println!("pmm: memory region {} of size {} bytes wll contain the pmm", containing_entry.0, containing_entry.1.length);

        // Create modules for all regions
        let mut root_module: Option<*mut PmmModule> = None;
        memory_regions.iter().filter(|entry| entry.entry_type == EntryType::USABLE).for_each(|entry| {
            serial_println!("pmm: creating module for region at 0x{:X}", entry.base);
            unsafe {
                let module_location = meta_buffer as *mut PmmModule;
                let bitmap_location = meta_buffer.add(size_of::<PmmModule>() * 2);

                // Set the root module or the last created module's next pointer
                match root_module {
                    None => {
                        let module = &mut *module_location;
                        root_module = Some(module);
                    },
                    Some(mut root) => {
                        let mut module = &mut *root;
                        while let Some(next) = module.next {
                            module = &mut *next;
                        }

                        module.next = Some(module_location);
                    }
                };

                let module = PmmModule::init(entry.base as PhysicalAddress, entry.length as usize, bitmap_location);
                let bitmap_size = module.bitmap_size;

                ptr::write(module_location, module);

                meta_buffer = meta_buffer.add(size_of::<PmmModule>() * 2 + bitmap_size);
            }
        });

        let mut allocator = Self {
            root_module: unsafe { &mut *root_module.unwrap() },
        };

        allocator.allocate_self_memory(containing_entry.0, buffer_size);

        Ok(allocator)
    }

    /// Allocate the memory used by the allocator itself such that it is not reallocated anywhere else
    fn allocate_self_memory(&mut self, containing_region_number: usize, buffer_size: usize) {
        let mut containing_module: &PmmModule = self.root_module;
        for _ in 0..containing_region_number {
            containing_module = unsafe { &mut *containing_module.next.unwrap() };
        }

        let frame_count = buffer_size.div_ceil(PAGE_SIZE);
        let byte_count = frame_count / 8;
        let bit_count = frame_count % 8;

        unsafe {
            for i in 0..byte_count {
                ptr::write(containing_module.bitmap.add(i), 0xFF);
            }

            ptr::write(containing_module.bitmap.add(byte_count), (1 << bit_count) - 1);
        }

    }
}
impl FrameAllocator for StaticLinearAllocator {
    fn allocate_frame(&mut self) -> Result<Frame, &'static str> {
        let mut module = unsafe { &mut *(self.root_module as *mut PmmModule) };
        loop {
            let alloc = module.allocate_frames(1);

            // Return the frame if it was found
            if let Some(alloc) = alloc {
                serial_println!("Allocating frame at address {:X}", alloc);
                let frame = Frame::containing_address(alloc);
                return Ok(frame);
            }
            // Try again with the next module if it exists, otherwise fail
            else {
                if let Some(next) = module.next {
                    module = unsafe { &mut *next };
                }
                else {
                    return Err("pmm: could not allocate frame (memory full)");
                }
            }
        }
    }

    fn deallocate_frame(&mut self, frame: Frame) -> Result<(), &'static str> {
        todo!()
    }
}