use core::alloc::Layout;
use core::ptr::NonNull;
use crate::{AllocError, AllocResult, BaseAllocator, PageAllocator, ByteAllocator};
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,

    total_pages: usize,
    used_pages: usize,
    page_pos: usize,

    total_bytes: usize,
    used_bytes: usize,
    byte_pos: usize,

}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
        
            total_pages: 0,
            used_pages: 0,
            page_pos: 0,
        
            total_bytes: 0,
            used_bytes: 0,
            byte_pos: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());
        self.start = start;
        self.end = start + size;
        let end_pg = super::align_down(start + size, PAGE_SIZE);
        let start_pg = super::align_up(start, PAGE_SIZE);

        self.page_pos = end_pg;
        self.byte_pos = start;

        self.total_pages = (end_pg - start_pg) / PAGE_SIZE;
        self.total_bytes = size;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        match num_pages.cmp(&1) {
            core::cmp::Ordering::Equal => Some(self.page_pos - PAGE_SIZE),
            core::cmp::Ordering::Greater => Some(self.page_pos - PAGE_SIZE * num_pages),
            _ => return Err(AllocError::InvalidParam),
        }
        .ok_or(AllocError::NoMemory)
        .inspect(|_| {
            self.used_pages += num_pages;
            self.page_pos -= PAGE_SIZE * num_pages;
        })
    }

    fn dealloc_pages(&mut self, _pos: usize, num_pages: usize) {
        // TODO: not decrease `used_pages` if deallocation failed
        self.used_pages -= num_pages;
        // self.inner.dealloc((pos - self.base) / PAGE_SIZE)
    }

    fn total_pages(&self) -> usize {
        self.total_pages
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        let end_pg = super::align_down(self.page_pos, PAGE_SIZE);
        let start_pg = super::align_up(self.byte_pos, PAGE_SIZE);
        (end_pg - start_pg) / PAGE_SIZE
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let result = NonNull::new(self.byte_pos as *mut u8);
        if let Some(res) = result {
            self.used_bytes += size;
            self.byte_pos += size;
            return Ok(res);
        } else {
            return Err(AllocError::NoMemory);
        }
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, layout: Layout) {
        let size = layout.size();
        self.used_bytes -= size;
        if self.used_bytes == 0 {
            self.byte_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    fn available_bytes(&self) -> usize {
        self.page_pos - self.byte_pos
    }
}