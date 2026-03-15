//! WASM linear memory allocation helpers.
//!
//! The host calls `atlas_alloc` before writing input bytes into the guest's
//! linear memory, and calls `atlas_dealloc` after reading the guest's output.
//! These `_impl` functions are called by the `export_source!` macro which
//! generates the actual `#[no_mangle]` exports in the source crate.

use std::alloc::Layout;

/// Allocate `size` bytes and return a pointer into the guest's linear memory.
///
/// # Safety
/// Called by the host runtime only. The returned pointer is valid until
/// `atlas_dealloc_impl` is called with the same pointer and size.
pub unsafe fn atlas_alloc_impl(size: u32) -> *mut u8 {
    if size == 0 {
        return core::ptr::NonNull::dangling().as_ptr();
    }
    let layout = Layout::from_size_align_unchecked(size as usize, 1);
    std::alloc::alloc(layout)
}

/// Free a buffer previously allocated by `atlas_alloc_impl`.
///
/// # Safety
/// `ptr` must be a pointer previously returned by `atlas_alloc_impl` with the
/// same `size` value.
pub unsafe fn atlas_dealloc_impl(ptr: *mut u8, size: u32) {
    if size == 0 {
        return;
    }
    let layout = Layout::from_size_align_unchecked(size as usize, 1);
    std::alloc::dealloc(ptr, layout);
}
