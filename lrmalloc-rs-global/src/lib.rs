use lrmalloc_rs::{do_aligned_alloc, do_free, do_realloc};
use std::os::raw::c_void;

/// Checks if a call to `malloc` use the lrmalloc-rs implementation.
///
/// Only works after `malloc` has been called at least once.
pub static mut OVERRIDE_MALLOC: bool = false;
/// Checks if a call to `calloc` use the lrmalloc-rs implementation.
///
/// Only works after `calloc` has been called at least once.
pub static mut OVERRIDE_CALLOC: bool = false;
/// Checks if a call to `realloc` use the lrmalloc-rs implementation.
///
/// Only works after `realloc` has been called at least once.
pub static mut OVERRIDE_REALLOC: bool = false;
/// Checks if a call to `free` use the lrmalloc-rs implementation.
///
/// Only works after `free` has been called at least once.
pub static mut OVERRIDE_FREE: bool = false;
/// Checks if a call to `aligned_alloc` use the lrmalloc-rs implementation.
///
/// Only works after `aligned_alloc` has been called at least once.
pub static mut OVERRIDE_ALIGNED_ALLOC: bool = false;

///Allocates size bytes of uninitialized storage.
///
/// If allocation succeeds, returns a pointer that is suitably aligned for any object type with fundamental alignment.
///
/// If size is zero, a pointer to the minimum sized allocation is created
#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
    unsafe {
        OVERRIDE_MALLOC = true;
    }
    do_aligned_alloc(8, size) as *mut c_void
}

/// Allocates memory for an array of num objects of size and initializes all bytes in the allocated storage to zero.
///
/// If allocation succeeds, returns a pointer to the lowest (first) byte in the allocated memory block that is suitably aligned for any object type.
///
/// If size is zero, a pointer to the minimum sized allocation is created
#[no_mangle]
pub extern "C" fn calloc(num: usize, size: usize) -> *mut c_void {
    unsafe {
        OVERRIDE_CALLOC = true;
    }
    let ret = malloc(num * size) as *mut u8;
    unsafe {
        for i in 0..(num * size) {
            *ret.offset(i as isize) = 0;
        }
    }
    ret as *mut c_void
}
/// Reallocates the given area of memory. It must be previously allocated by malloc(), calloc() or realloc() and not yet freed with a call to free or realloc. Otherwise, the results are undefined.
///
/// The reallocation is done by either:
///
/// 1. Allocating a new memory block of size new_size bytes, copying memory area with size equal the lesser of the new and the old sizes, and freeing the old block.
/// If there is not enough memory, the old memory block is not freed and null pointer is returned.
/// 2. Keeping the block in the same space, if the size class of the new size is the same
///
/// If ptr is NULL, the behavior is the same as calling malloc(new_size).
///
/// If size is zero, a pointer to the minimum sized allocation is created
#[no_mangle]
pub extern "C" fn realloc(ptr: *mut c_void, new_size: usize) -> *mut c_void {
    unsafe {
        OVERRIDE_REALLOC = true;
    }
    do_realloc(ptr, new_size)
}

/// Deallocates the space previously allocated by malloc(), calloc(), aligned_alloc() or realloc().
///
/// If ptr is a null pointer, the function does nothing.
///
/// The behavior is undefined if the value of ptr does not equal a value returned earlier by malloc(), calloc(), realloc(), or aligned_alloc() (since C11).
///
/// The behavior is undefined if the memory area referred to by ptr has already been deallocated, that is, free() or realloc() has already been called with ptr as the argument and no calls to malloc(), calloc() or realloc() resulted in a pointer equal to ptr afterwards.
///
/// The behavior is undefined if after free() returns, an access is made through the pointer ptr (unless another allocation function happened to result in a pointer value equal to ptr)
#[no_mangle]
pub extern "C" fn free(ptr: *mut c_void) {
    unsafe {
        OVERRIDE_FREE = true;
    }
    do_free(ptr)
}

#[no_mangle]
pub extern "C" fn aligned_alloc(alignment: usize, size: usize) -> *mut c_void {
    unsafe {
        OVERRIDE_ALIGNED_ALLOC = true;
    }
    do_aligned_alloc(alignment, size) as *mut c_void
}

#[no_mangle]
pub extern "C" fn check_override() -> bool {
    unsafe {
        let ptr = malloc(8);
        if !OVERRIDE_MALLOC {
            return false;
        }
        let new_ptr = realloc(ptr, 64);
        assert_ne!(new_ptr, ptr);
        if !OVERRIDE_REALLOC {
            return false;
        }
        let calloced = calloc(8, 8);
        assert_ne!(new_ptr, calloced);
        if !OVERRIDE_CALLOC {
            return false;
        }
        do_free(new_ptr);
        do_free(calloced);
        if !OVERRIDE_FREE {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn overrides_malloc() {
        unsafe {
            OVERRIDE_MALLOC = false;
            let _ret = libc::malloc(8);
            assert!(OVERRIDE_MALLOC, "Malloc wasn't overwritten!")
        }
    }
    #[test]
    fn overrides_calloc() {
        unsafe {
            OVERRIDE_CALLOC = false;
            let _ret = libc::calloc(1, 8);
            assert!(OVERRIDE_CALLOC, "Calloc wasn't overwritten!")
        }
    }
    #[test]
    fn overrides_realloc() {
        unsafe {
            OVERRIDE_REALLOC = false;
            let first = libc::malloc(8);
            let _ret = libc::realloc(first, 8);
            assert!(OVERRIDE_REALLOC, "Realloc wasn't overwritten!")
        }
    }
    #[test]
    fn overrides_free() {
        unsafe {
            let ret = libc::malloc(8);
            OVERRIDE_FREE = false;
            libc::free(ret);
            assert!(OVERRIDE_FREE, "Free wasn't overwritten!")
        }
    }
}
