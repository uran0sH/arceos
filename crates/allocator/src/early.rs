use core::{alloc::Layout, ptr::NonNull};

use crate::AllocResult;

pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start_vaddr: usize,
    end_vaddr: usize,
    byte_pos: usize,
    page_pos: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start_vaddr: 0,
            end_vaddr: 0,
            byte_pos: 0,
            page_pos: PAGE_SIZE,
        }
    }

    pub fn init(&mut self, start_vaddr: usize, size: usize) {
        self.start_vaddr = super::align_up(start_vaddr, PAGE_SIZE);
        self.end_vaddr = super::align_down(start_vaddr + size, PAGE_SIZE);
        self.byte_pos = self.start_vaddr;
        self.page_pos = self.end_vaddr;
    }

    pub fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let ptr = NonNull::new(self.byte_pos as *mut u8).ok_or(crate::AllocError::NoMemory)?;
        self.byte_pos += layout.size();
        Ok(ptr)
    }

    pub fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {}

    pub fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        assert_eq!(align_pow2 % PAGE_SIZE, 0);
        self.page_pos -= num_pages * PAGE_SIZE;
        Ok(self.page_pos)
    }

    pub fn dealloc_pages(&self, _pos: usize, _num_pages: usize) {}

    pub fn used_bytes(&self) -> usize {
        self.byte_pos - self.start_vaddr
    }

    pub fn available_bytes(&self) -> usize {
        assert!(self.byte_pos <= self.page_pos);
        self.page_pos - self.byte_pos
    }

    pub fn used_pages(&self) -> usize {
        (self.end_vaddr - self.page_pos) / PAGE_SIZE
    }

    pub fn available_pages(&self) -> usize {
        assert!(self.byte_pos <= self.page_pos);
        (self.page_pos - self.byte_pos) / PAGE_SIZE
    }
}
