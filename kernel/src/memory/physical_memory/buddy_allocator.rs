use alloc::collections::LinkedList;
use alloc::vec::Vec;
use core::cmp::min;
use limine::memory_map::{Entry, EntryType};
use limine::response::MemoryMapResponse;
use crate::memory::{Frame, PAGE_SIZE, PhysicalAddress};
use crate::memory::physical_memory::FrameAllocator;

// Linker script symbols marking ELF sections
extern "C" {
    static _stext: u8;
    static _etext: u8;
    static _srodata: u8;
    static _erodata: u8;
    static _sdata: u8;
    static _edata: u8;
    static _sdynamic: u8;
    static _edynamic: u8;
    static _sbss: u8;
    static _ebss: u8;
}

// Maximum allocation size, this allocator cannot allocate blocks larger than 2^MAX_ORDER pages
const MAX_ORDER: usize = 10;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum BlockType {
    TopLevel,
    LeftBuddy,
    RightBuddy
}

type MemoryBlocks = [LinkedList<MemoryBlock>; MAX_ORDER + 1];
pub struct BuddyAllocator {
    memory_blocks: MemoryBlocks,
    allocated_amount: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct MemoryBlock {
    is_allocated: bool,
    starting_address: PhysicalAddress,
    size_class: usize,
    block_type: BlockType
}

impl MemoryBlock {
    fn contains_address(&self, address: PhysicalAddress) -> bool {
        address >= self.starting_address && address < self.starting_address + PAGE_SIZE * 2usize.pow(self.size_class as u32)
    }
}

impl BuddyAllocator {
    pub fn new(memory_map: &'static MemoryMapResponse) -> Self {
        let mut memory_blocks: MemoryBlocks = [
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
            LinkedList::new(),
        ];

        // Fill the memory block lists
        for area in memory_map.entries().iter().filter(|entry| entry.entry_type == EntryType::USABLE) {
            Self::map_area(area, &mut memory_blocks);
        }

        Self {
            memory_blocks,
            allocated_amount: 0,
        }
    }

    /// Returns the total amount of memory allocated by this allocator
    pub fn get_allocated_amount(&self) -> usize {
        self.allocated_amount
    }

    pub fn display_memory(&self) {
        println!("{:?}", self.memory_blocks);
    }

    /// Marks the specified frames as allocated. This is mostly used when transitioning
    /// from the linear allocator to this one, but it is also occasionally used when hardware
    /// specifies certain address like the AHCI controller
    pub fn set_allocated_frames(&mut self, frames: Vec<PhysicalAddress>) -> Result<(), &'static str> {
        for frame_address in frames {
            if let Err(err) = self.allocate_frame_at_address(frame_address) {
                return Err(&*err);
            }
        }

        Ok(())
    }

    /// Allocates 2^order contiguous frames
    /// Returns the starting address of the allocated block
    pub fn allocate_frames(&mut self, order: usize) -> Result<PhysicalAddress, &'static str> {
        if order > MAX_ORDER {
            return Err("cannot allocate more than 10 contiguous frames")
        }

        let first_free_block = self.memory_blocks[order].iter_mut().find(|block| !block.is_allocated);
        if first_free_block.is_some() {
            let block = first_free_block.unwrap();
            block.is_allocated = true;

            self.allocated_amount += 2usize.pow(order as u32) * PAGE_SIZE;
            Ok(block.starting_address)
        } else {
            let alloc = self.split_block(order + 1);

            if alloc.is_ok() {
                self.allocated_amount += 2usize.pow(order as u32) * PAGE_SIZE;
            }

            alloc
        }
    }

    /// Deallocates 2^order contiguous frames
    pub fn deallocate_frames(&mut self, start_address: PhysicalAddress, order: usize) -> Result<(), &'static str> {
        let memory_block = self.memory_blocks[order].iter_mut()
            .find(|block| block.starting_address == start_address);

        if memory_block.is_none() {
            return Err("could not find the frame to deallocate");
        }

        if let Some(memory_block) = memory_block {
            if !memory_block.is_allocated {
                return Err("frame was already unallocated");
            }

            memory_block.is_allocated = false;

            // Merge only if block is a buddy
            if memory_block.block_type == BlockType::TopLevel {
                self.allocated_amount -= 2usize.pow(order as u32) * PAGE_SIZE;
                return Ok(());
            }

            let buddy_address = if memory_block.block_type == BlockType::LeftBuddy {
                memory_block.starting_address + PAGE_SIZE * 2usize.pow(memory_block.size_class as u32)
            } else {
                memory_block.starting_address - PAGE_SIZE * 2usize.pow(memory_block.size_class as u32)
            };

            let buddy = self.memory_blocks[order].iter_mut()
                .find(|block| block.starting_address == buddy_address);

            if buddy.is_none() {
                return Err("could not find the frame to deallocate");
            }

            // Merge the two blocks
            if let Some(buddy) = buddy {
                if !buddy.is_allocated {
                    let parent_block_address = min(start_address, buddy_address);

                    let _extracted_buddy = self.memory_blocks[order]
                        .extract_if(|block| block.starting_address == start_address);
                    let _extracted_buddy = self.memory_blocks[order]
                        .extract_if(|block| block.starting_address == buddy_address);

                    let parent_block = self.memory_blocks[order + 1]
                        .iter_mut()
                        .find(|block| block.starting_address == parent_block_address);

                    match parent_block {
                        Some(parent_block) => parent_block.is_allocated = false,
                        None => return Err("could not find a parent block")
                    }
                }
            }
        }

        self.allocated_amount -= 2usize.pow(order as u32) * PAGE_SIZE;
        Ok(())
    }

    /// Allocates a single frame at a given address. This is mostly used when transitioning from
    /// the linear allocator to this one.
    fn allocate_frame_at_address(&mut self, address: PhysicalAddress) -> Result<PhysicalAddress, &'static str> {
        if self.memory_blocks[0].iter().any(|block| block.is_allocated && block.starting_address == address) {
            return Err("frame already allocated");
        }

        // 1. Find the biggest free block containing the address
        let mut current_block: Option<&mut MemoryBlock> = None;
        let mut current_order = 0;
        while current_block.is_none() && current_order <= MAX_ORDER {
            current_block = self.memory_blocks[current_order].iter_mut().find(|block| block.contains_address(address));
            current_order += 1;
        }

        let current_block = current_block.expect("could not allocate memory");
        current_block.is_allocated = true;

        let mut current_block_clone = *current_block;

        while current_block_clone.size_class > 0 {
            let buddy_size_class = current_block_clone.size_class - 1;

            let mut left_buddy = MemoryBlock {
                is_allocated: false,
                starting_address: current_block_clone.starting_address,
                size_class: buddy_size_class,
                block_type: BlockType::LeftBuddy
            };

            let mut right_buddy = MemoryBlock {
                is_allocated: false,
                starting_address: current_block_clone.starting_address + PAGE_SIZE * 2usize.pow(buddy_size_class as u32),
                size_class: buddy_size_class,
                block_type: BlockType::RightBuddy,
            };

            if left_buddy.contains_address(address) {
                left_buddy.is_allocated = true;
                current_block_clone = left_buddy;

                self.memory_blocks[buddy_size_class].push_back(left_buddy);
                self.memory_blocks[buddy_size_class].push_back(right_buddy);
            }
            else {
                right_buddy.is_allocated = true;
                current_block_clone = right_buddy;

                self.memory_blocks[buddy_size_class].push_back(left_buddy);
                self.memory_blocks[buddy_size_class].push_back(right_buddy);
            }
        }

        self.allocated_amount += PAGE_SIZE;
        Ok(current_block_clone.starting_address)
    }

    fn map_area(area: &Entry, memory_blocks: &mut MemoryBlocks) {
        let mut block_start_address = area.base as PhysicalAddress;
        let mut block_end_address = block_start_address + PAGE_SIZE * 2usize.pow(MAX_ORDER as u32);

        while block_start_address < (area.base + area.length) as PhysicalAddress {
            let mut current_order = MAX_ORDER as u32;

            // Find the largest block that fits
            while block_end_address > (area.base + area.length) as PhysicalAddress {
                // If no block order fits, no more blocks can be added for this area
                if current_order == 0 {
                    return;
                }

                current_order -= 1;
                block_end_address = block_start_address + PAGE_SIZE * 2usize.pow(current_order);
            }

            // Add the block to its corresponding list
            memory_blocks[current_order as usize].push_back(MemoryBlock {
                is_allocated: false,
                starting_address: block_start_address,
                size_class: current_order as usize,
                block_type: BlockType::TopLevel
            });

            // Move on to the next block
            block_start_address += (PAGE_SIZE * 2usize.pow(current_order)) as PhysicalAddress;
            block_end_address = block_start_address + PAGE_SIZE * 2usize.pow(MAX_ORDER as u32);
        }
    }

    /// Split a 2^order sized block into two 2^order-1 sized blocks, and sets the first one as allocated and returns it.
    /// The created blocks are added to the free_areas array at index order-1 and the original block is marked as allocated.
    fn split_block(&mut self, order: usize) -> Result<PhysicalAddress, &'static str> {
        if order == 0 {
            return Err("cannot split block further");
        }

        // Find the first, smallest unallocated block that fits
        let mut first_free_block: Option<&mut MemoryBlock> = None;
        let mut current_order = order;
        while first_free_block.is_none() && current_order <= MAX_ORDER {
            first_free_block = self.memory_blocks[current_order].iter_mut().find(|block| !block.is_allocated);
            current_order += 1;
        }

        match first_free_block {
            Some(current_block) => {
                current_block.is_allocated = true;

                let mut current_block_clone = *current_block;

                // Repeatedly split until we get to the desired size
                while current_block_clone.size_class >= order {
                    let buddy_size_class = current_block_clone.size_class - 1;

                    let left_buddy = MemoryBlock {
                        is_allocated: true,
                        starting_address: current_block_clone.starting_address,
                        size_class: buddy_size_class,
                        block_type: BlockType::LeftBuddy
                    };

                    let right_buddy = MemoryBlock {
                        is_allocated: false,
                        starting_address: current_block_clone.starting_address + PAGE_SIZE * 2usize.pow(buddy_size_class as u32),
                        size_class: buddy_size_class,
                        block_type: BlockType::RightBuddy
                    };

                    // Add the two buddies to the linked list
                    self.memory_blocks[buddy_size_class].push_back(left_buddy);
                    self.memory_blocks[buddy_size_class].push_back(right_buddy);

                    // Return only the (allocated) left buddy
                    current_block_clone = left_buddy
                }

                Ok(current_block_clone.starting_address)
            },
            None => Err("encountered an error while splitting block")
        }
    }
}

impl FrameAllocator for BuddyAllocator {
    fn allocate_frame(&mut self) -> Result<Frame, &'static str> {
        let frame_address = self.allocate_frames(0)?;
        let frame = Frame::containing_address(frame_address);

        Ok(frame)
    }

    fn deallocate_frame(&mut self, frame: Frame) -> Result<(), &'static str> {
        self.deallocate_frames(frame.start_address(), 0)
    }
}

#[cfg(test)]
mod tests {
    use limine::memory_map::EntryType;
    use crate::memory::PAGE_SIZE;
    use crate::memory::physical_memory::buddy_allocator::{BlockType, BuddyAllocator, MAX_ORDER, MemoryBlock};
    use crate::memory::physical_memory::FrameAllocator;
    use crate::MEMORY_MAP_REQUEST;

    #[test_case]
    fn allocation_too_large() {
        // GIVEN
        let memory_map = MEMORY_MAP_REQUEST.get_response().expect("could not find the memory map");
        let mut allocator = BuddyAllocator::new(memory_map);

        // WHEN
        let result = allocator.allocate_frames(MAX_ORDER + 1);

        // THEN
        assert!(result.is_err());
        //assert_eq!(result.unwrap(), "cannot allocate more than 10 contiguous frames");
    }

    #[test_case]
    fn allocate_frame_happy_path() {
        // GIVEN
        let memory_map = MEMORY_MAP_REQUEST.get_response().expect("could not find the memory map");
        let mut allocator = BuddyAllocator::new(memory_map);

        // WHEN
        let frame = allocator.allocate_frame();
        let frame_start = frame.unwrap().start_address();
        let containing_region = memory_map.entries().iter()
            .filter(|entry| entry.base < frame_start as u64 && entry.base + entry.length > frame_start as u64)
            .next().expect("could not find the region containing the allocation");

        // THEN
        assert!(frame.is_ok()); // The frame was allocated correctly
        assert!(matches!(containing_region.entry_type, EntryType::USABLE)); // The frame is in a usable region
    }

    #[test_case]
    fn allocate_multiple_frames_no_overlap() {
        // GIVEN
        let memory_map = MEMORY_MAP_REQUEST.get_response().expect("could not find the memory map");
        let mut allocator = BuddyAllocator::new(memory_map);

        // WHEN
        let first_frame = allocator.allocate_frame();
        let second_frame = allocator.allocate_frame();

        // THEN
        assert_ne!(first_frame.unwrap().start_address(), second_frame.unwrap().start_address());
    }

    #[test_case]
    fn reuse_old_frame_after_deallocating() {
        // GIVEN
        let memory_map = MEMORY_MAP_REQUEST.get_response().expect("could not find the memory map");
        let mut allocator = BuddyAllocator::new(memory_map);

        // WHEN
        let first_frame = allocator.allocate_frame();
        allocator.deallocate_frame(first_frame.unwrap()).expect("could not deallocate");
        let second_frame = allocator.allocate_frame();

        // THEN
        assert_eq!(first_frame.unwrap().start_address(), second_frame.unwrap().start_address());
    }

    #[test_case]
    fn split_block_happy_path() {
        // GIVEN
        let memory_map = MEMORY_MAP_REQUEST.get_response().expect("could not find the memory map");

        let large_block_size = 5;
        let large_block = MemoryBlock {
            is_allocated: false,
            starting_address: 0,
            size_class: large_block_size,
            block_type: BlockType::TopLevel
        };
        let expected_left_buddy = MemoryBlock {
            is_allocated: true,
            starting_address: 0,
            size_class: large_block_size - 1,
            block_type: BlockType::LeftBuddy
        };
        let expected_right_buddy = MemoryBlock {
            is_allocated: false,
            starting_address: 2usize.pow((large_block_size - 1) as u32) * PAGE_SIZE,
            size_class: large_block_size - 1,
            block_type: BlockType::RightBuddy
        };

        let mut allocator = BuddyAllocator::new(memory_map);
        allocator.memory_blocks[large_block_size].push_front(large_block);

        // WHEN
        let split_block = allocator.split_block(5);

        // THEN
        assert!(split_block.is_ok());
        assert_eq!(allocator.memory_blocks[large_block_size - 1].contains(&expected_left_buddy), true);
        assert_eq!(allocator.memory_blocks[large_block_size - 1].contains(&expected_right_buddy), true);
    }
}