mod slab_allocator;

use crate::memory::{VirtualAddress};
use crate::memory::virtual_memory::heap_allocator::slab_allocator::SlabAllocator;
use crate::memory::virtual_memory::paging::{ActivePageTable, Page};
use crate::memory::virtual_memory::paging::entry::EntryFlags;
use crate::memory::physical_memory::FrameAllocator;

pub const HEAP_START: VirtualAddress = 0xFFFFC90000000000;
pub const HEAP_SIZE: usize = 1000 * 1024; // 1 MiB

#[global_allocator]
pub static ALLOCATOR: Locked<SlabAllocator> = Locked::new(SlabAllocator::new());

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub fn init_heap<A>(frame_allocator: &mut A, page_table: &mut ActivePageTable) where A: FrameAllocator {
    serial_println!("mm: initializing the heap...");

    let page_range = {
        let heap_start: VirtualAddress = HEAP_START;
        let heap_end: VirtualAddress = heap_start + HEAP_SIZE - 1usize;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    serial_println!("Heap from {:X} to {:X}", HEAP_START, HEAP_START + HEAP_SIZE);

    for page in page_range {
        let frame = frame_allocator.allocate_frame().expect("Frame allocation failed");

        let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE;

        page_table.map_to(page, frame, flags, frame_allocator)
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    serial_println!("mm: heap starts at 0x{:X}", HEAP_START);
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use crate::memory::MemoryManager;
    use crate::memory::virtual_memory::heap_allocator::HEAP_SIZE;
    use crate::MEMORY_MAP_REQUEST;

    fn init() {
        MemoryManager::init(MEMORY_MAP_REQUEST.get_response().expect("could not retrieve the memory map")).expect("whoopsy daisy");
    }

    #[test_case]
    fn box_allocation() {
        init();

        let heap_value_1 = Box::new(41);
        let heap_value_2 = Box::new(13);

        assert_eq!(*heap_value_1, 41);
        assert_eq!(*heap_value_2, 13);
    }

    #[test_case]
    fn large_vec() {
        init();

        let n =1000;
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(i);
        }

        assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
    }

    /*
    #[test_case]
    fn many_boxes() {
        for i in 0..HEAP_SIZE {
            let x = Box::new(i);

            assert_eq!(*x, i);
        }
    }*/
}