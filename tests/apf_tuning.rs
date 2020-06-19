extern crate lrmalloc_rs;

use lrmalloc_rs::{ do_aligned_alloc, do_free };
use std::alloc::{ GlobalAlloc, Layout };
use std::thread;
use core::sync::atomic::Ordering;

struct Apf;

unsafe impl GlobalAlloc for Apf {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		do_aligned_alloc(layout.align(), layout.size())
	}

	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
		do_free(ptr);
	}
}

#[global_allocator]
static ALLOCATOR: Apf = Apf;

#[test]
fn test_apf_tuning() {
	let mut vec = vec![];

	for i in 0..30 {
		vec.push(thread::spawn(move || {
			//println!("Thread {}", &i);
			let mut boxes = Vec::<Box::<usize>>::new();
			for j in 0..10 {
				boxes.push(Box::new(j));
				println!("Thread {} made box {}", i, j);
			}
		}));
	}

	for join_handle in vec {
        join_handle.join().unwrap();
    }

    println!("test");
    println!("{}", lrmalloc_rs::thread_cache::apf_init.with(|init| { *init.borrow() }));

    println!(
        "Allocated in bootstrap: {} bytes",
        lrmalloc_rs::IN_BOOTSTRAP.load(Ordering::Relaxed)
    );

    println!(
        "Allocated in cache: {} bytes",
        lrmalloc_rs::IN_CACHE.load(Ordering::Relaxed)
    );
    //panic!();

}