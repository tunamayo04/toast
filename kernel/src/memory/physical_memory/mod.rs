use crate::memory::{PAGE_SIZE, PhysicalAddress};

pub mod linear_frame_allocator;
pub mod buddy_allocator;
mod static_buddy_allocator;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Frame {
    pub(super) number: usize
}

impl Frame {
    /// Returns the frame containing the address passed as argument
    pub fn containing_address(address: PhysicalAddress) -> Frame {
        Frame{ number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start,
            end
        }
    }

    fn clone(&self) -> Frame {
        Frame { number: self.number }
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        }
        else {
            None
        }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Result<Frame, &'static str>;
    fn deallocate_frame(&mut self, frame: Frame) -> Result<(), &'static str>;
}