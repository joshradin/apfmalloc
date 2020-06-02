#![allow(non_upper_case_globals)]


use std::sync::atomic::AtomicPtr;
use crate::allocation_data::{DescriptorNode, ProcHeap, get_heaps, Descriptor, SuperBlockState, Anchor};
use std::ptr::null_mut;
use crate::mem_info::{MAX_SZ_IDX, MAX_SZ};
use lazy_static::lazy_static;
use crossbeam::atomic::AtomicCell;
use std::cell::{Cell, RefCell};
use crate::size_classes::{init_size_class, get_size_class, SIZE_CLASSES};
use std::process::id;
use crate::page_map::S_PAGE_MAP;
use std::borrow::Borrow;
use memmap::MmapMut;
use atomic::{Ordering, Atomic};
use crate::pages::{page_alloc, page_free};
use crate::alloc::{register_desc, fill_cache, get_page_info_for_ptr, unregister_desc, flush_cache};

#[macro_use] pub mod macros;
mod size_classes;
mod mem_info;
mod allocation_data;
mod pages;
mod page_map;
mod thread_cache;
mod alloc;

#[macro_use]
extern crate bitfield;

lazy_static! {
    static ref AVAILABLE_DESC: Atomic<DescriptorNode> = Atomic::new(DescriptorNode::new());
}
static mut MALLOC_INIT: bool = false;



unsafe fn init_malloc() {
    MALLOC_INIT = true;
    init_size_class();

    S_PAGE_MAP.init();

    for idx in 0..MAX_SZ_IDX {
        let heap = get_heaps().get_heap_at_mut(idx);

        heap.partial_list.store(
            None, Ordering::Release
        );
        heap.size_class_index = idx;

    }
}

unsafe fn thread_local_init_malloc() {
    /*
    if thread_cache::thread_init.with(|f| *f.borrow()) {

    }

     */
    thread_cache::thread_init.with(|f| {
        let mut ref_mut = f.borrow_mut();
        *ref_mut = true;
    });

    thread_cache::thread_cache.with(|f| {

    });
}

pub fn do_malloc(size: usize) -> *mut u8{
    unsafe {
        if !MALLOC_INIT {
            init_malloc();
        }
    }


    if size > MAX_SZ {
        let pages = page_ceiling!(size);
        let desc = unsafe { &mut *Descriptor::alloc() };

        desc.proc_heap = null_mut();
        desc.block_size = pages as u32;
        desc.max_count = 1;
        desc.super_block = page_alloc(pages).expect("Should create");

        let mut anchor = Anchor::default();
        anchor.set_state(SuperBlockState::FULL);

        desc.anchor.store(anchor, Ordering::Acquire);

        register_desc(desc);
        let ptr = desc.super_block;
        return ptr;
    }

    let size_class_index = get_size_class(size);

    thread_cache::thread_cache.with(
        |tcache| {
            let cache = &mut tcache.borrow_mut()[size_class_index];
            if cache.get_block_num() == 0 {
                fill_cache(size_class_index, cache);
            }

            cache.pop_block()
        }
    )


}

fn is_power_of_two(x: usize) -> bool {
    // https://stackoverflow.com/questions/3638431/determine-if-an-int-is-a-power-of-2-or-not-in-a-single-line
    (if x != 0 { true } else { false }) && (if (!(x & (x -1))) != 0 {
        true
    } else {
        false
    })
}

pub fn do_free<T>(ptr: * const T) {
    let info = get_page_info_for_ptr(ptr);
    let desc = unsafe { &mut *info.get_desc().expect("descriptor should exist here") };

    let size_class_index = info.get_size_class_index();
    match size_class_index {
        None => {
            let super_block = desc.super_block;
            // unregister
            unregister_desc(None, super_block);

            // if large allocation
            if ptr as * const u8 != super_block as * const u8 {
                unregister_desc(None, ptr as * mut u8)
            }

            // free the super block
            page_free(super_block);

            // retire the descriptor
            desc.retire();
        },
        Some(size_class_index) => {
            thread_cache::thread_cache.with(
                |tcache| {
                    let cache = &mut tcache.borrow_mut()[size_class_index];
                    let sc = unsafe {& SIZE_CLASSES[size_class_index]};

                    if cache.get_block_num() >= sc.cache_block_num {
                        flush_cache(size_class_index, cache);
                    }

                    cache.push_block(ptr as * mut u8);
                }
            )
        },
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocation_data::get_heaps;
    use bitfield::size_of;

    #[test]
    fn heaps_valid() {
        let heap = get_heaps();
        let p_heap = heap.get_heap_at_mut(0);

    }

    #[test]
    fn malloc_and_free() {
        let ptr = unsafe { &mut *(super::do_malloc(size_of::<usize>()) as *mut usize)};
        *ptr = 8;
        assert_eq!(ptr, &8); // should be trivial
        do_free(ptr as * mut usize);
    }
}
