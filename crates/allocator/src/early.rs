

use core::ptr::NonNull;
use rlsf::Tlsf;
use crate::{AllocError, AllocResult, BaseAllocator, PageAllocator,ByteAllocator};

// 使用bitmap分配器所用的位图工具
use bitmap_allocator::BitAlloc;
type BitAllocUsed = bitmap_allocator::BitAlloc1M;

// 注：并没有考虑两者地址分配重合的情况
// 因此如果分配的太多，导致地址重合，会出错。
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    // ---------页分配----------------
    start:usize,//地址段的起始位置，此处是字节分配器使用
    end:usize,//地址段的终止位置，此处是页分配器使用
    total_pages: usize,//总共的页数
    used_pages: usize,//使用到的页数
    pages_inner: BitAllocUsed,//位图记录页的分配，需要注意的是对下标进行了转换
    // ---------字节分配---------------
    total_bytes: usize,//总共的字节数
    used_bytes: usize,//使用到的字节数
    bytes_inner: Tlsf<'static, u32, u32, 28, 32>, // max pool size: 32 * 2^28 = 8G
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    /// Creates a new empty `BitmapPageAllocator`.
    pub const fn new() -> Self {
        Self {
            start:0,
            end:0,
            total_pages:0,
            used_pages:0,
            pages_inner:BitAllocUsed::DEFAULT,
            
            total_bytes: 0,
            used_bytes: 0,
            bytes_inner: Tlsf::new(),
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE>  {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());
        
        let end = super::align_down(start + size, PAGE_SIZE);
        let start = super::align_up(start, PAGE_SIZE);
        (self.start,self.end) = (start,end);
        // 初始化页分配
        self.total_pages = (end - start) / PAGE_SIZE;
        self.pages_inner.insert(0..self.total_pages);

        unsafe {
            let pool = core::slice::from_raw_parts_mut(start as *mut u8, size);
            self.bytes_inner
                .insert_free_block_ptr(NonNull::new(pool).unwrap())
                .unwrap();
        }
        self.total_bytes = size;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        unimplemented!()   
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
        let align_log2 = align_pow2.trailing_zeros() as usize;
        match num_pages.cmp(&1) {
            //因为从后往前分配，因此此处的idx需要+1
            core::cmp::Ordering::Equal => self.pages_inner.alloc().map(|idx| self.end-(idx+1) * PAGE_SIZE), 
            core::cmp::Ordering::Greater => self
                .pages_inner
                .alloc_contiguous(num_pages, align_log2)
                .map(|idx| self.end-(idx+1) * PAGE_SIZE),
            _ => return Err(AllocError::InvalidParam),
        }
        .ok_or(AllocError::NoMemory)
        .inspect(|_| self.used_pages += num_pages)
    }
    fn available_pages(&self) -> usize {
        unimplemented!()
    }
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        self.used_pages -= num_pages;
        // 因为是从后往前分配，因此用self.end-pos
        self.pages_inner.dealloc((self.end-pos) / PAGE_SIZE);
    }
    fn total_pages(&self) -> usize {
        unimplemented!()
    }
    fn used_pages(&self) -> usize {
        self.used_pages
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: core::alloc::Layout) -> AllocResult<core::ptr::NonNull<u8>> {
        let ptr = self.bytes_inner.allocate(layout).ok_or(AllocError::NoMemory)?;
        self.used_bytes += layout.size();
        Ok(ptr)
    }
    fn available_bytes(&self) -> usize {
        unimplemented!()
    }
    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { self.bytes_inner.deallocate(pos, layout.align()) }
        self.used_bytes -= layout.size();
    }
    fn total_bytes(&self) -> usize {
        unimplemented!()
    }
    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

}