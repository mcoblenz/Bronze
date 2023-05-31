#![feature(bronze_gc)]

use std::cell::Cell;
use std::rc::Rc;
use backtrace::Backtrace;
use std::borrow::{Borrow, BorrowMut};
use std::path::PathBuf;

// Tests must be run sequentially because the shadow stack implementation does not support concurrency.
// Unfortunately Rust doesn't support configuring tests to run with only one thread, so we have to use #[serial] on every test!
// https://github.com/rust-lang/rust/issues/43155
use bronze_gc::*;
use bronze_derive::{Trace, Finalize};
use serial_test::serial;

mod dynamic_borrowing;

#[test]
#[serial]
fn new_ref() {
    let num_ref: GcRef<i32> = Gc::new(42);
    let ref_alias = num_ref;
    let gc_num_ref = ref_alias.borrow();

    // GcRef is Copy, so this is fine.
    let gc_num_ref2 = num_ref.borrow();

    assert_eq!(*gc_num_ref, 42);
    assert_eq!(*gc_num_ref2, 42);
}


// TODO: Re-enable these trait object tests.
// pub trait Shape {}
// struct Square {}
// impl Shape for Square {}

// unsafe impl GcTrace for Square {
//     unsafe fn trace(&self) {}
// }

// unsafe impl GcTrace for dyn Shape {
//     unsafe fn trace(&self) {}
// }

// fn take_shape(_shape: GcRef<dyn Shape>) {}

// fn take_shape_box(_shape: Box<dyn Shape>) {}

// #[test]
// fn unsize_test() {
//     let sq = Square{};

//     let gc_square = Gc::new(sq);
//     take_shape(gc_square);
// }

// #[test]
// fn box_test() {
//     let sq = Square{};
//     let b = Box::new(sq);

//     take_shape_box(b);
// }



#[test]
#[serial]
fn boxes() {
    assert_eq!(boxes_len(), 0);
    let _num_gc_ref_1 = Gc::new(42);
    assert_eq!(boxes_len(), 1);
    // println!("first ref: {:p}", _num_gc_ref_1.obj_ref);
    let _num_gc_ref_2 = Gc::new(42);
    assert_eq!(boxes_len(), 2);
    // println!("second ref: {:p}", _num_gc_ref_2.obj_ref);
}

#[derive(Trace, Finalize)]
struct OneRef {
    r: GcRef<i32>,
}

#[test]
#[serial]
fn one_ref() {
    assert_eq!(boxes_len(), 0);
    let num_gc_ref_1 = Gc::new(42);
    let _one_ref_1 = Gc::new(OneRef{r: num_gc_ref_1});
    let num_gc_ref_2 = Gc::new(42);
    let _one_ref_2 = Gc::new(OneRef{r: num_gc_ref_2});
    assert_eq!(boxes_len(), 4);
}

fn alloc_one_num() {
    let _num_gc_ref_1 = Gc::new(42);
}

#[test]
#[serial]
fn collect_one_ref() {
    assert_eq!(boxes_len(), 0);
    alloc_one_num();
}

#[test]
#[serial]
fn collect_two_refs() {
    assert_eq!(boxes_len(), 0);
    alloc_one_num();
    alloc_one_num();

    // At this point, the stack map should show that the first ref is not a root.
    // Therefore, it should get collected in the next collection.
    let _num_gc_ref_2 = Gc::new(42); // Should NOT get collected.
}


#[derive(Trace)]
struct TrackedAllocation {
    tracker: Rc<Cell<u32>>,
}

impl TrackedAllocation {
    fn new(tracker: Rc<Cell<u32>>) -> GcRef<TrackedAllocation> {
        tracker.set(tracker.as_ref().get() + 1);
        println!("initialized one tracked allocation. Count is now {}", tracker.as_ref().get());

        Gc::new(TrackedAllocation {tracker})
    }
}

impl Finalize for TrackedAllocation {
    fn finalize(&self) {
        self.tracker.set(self.tracker.as_ref().get() - 1);
        println!("finalized one tracked allocation. Count is now {}", self.tracker.as_ref().get());
        // let bt = Backtrace::new();
        // println!("{:?}", bt);
        
    }
}

fn alloc_one_tracked_obj(tracker: Rc<Cell<u32>>) -> GcRef<TrackedAllocation> {
    TrackedAllocation::new(tracker)
}


#[test]
#[serial]
fn one_allocation() {
    let outstanding_allocations = Rc::new(Cell::new(0));
    assert_eq!(outstanding_allocations.as_ref().get(), 0);
    n_allocations(1, outstanding_allocations.clone());
    assert_eq!(outstanding_allocations.as_ref().get(), 1);
}

fn n_allocations(n: u32, tracker: Rc<Cell<u32>>) {
    println!("n_allocations start");
    let mut objs = Vec::new();

    for _i in 0..n {
        objs.push(TrackedAllocation::new(tracker.clone()));
    }
    println!("n_allocations end");

}

struct TenRefs {
    r1: GcRef<TrackedAllocation>,
    r2: GcRef<TrackedAllocation>,
    r3: GcRef<TrackedAllocation>,
    r4: GcRef<TrackedAllocation>,
    r5: GcRef<TrackedAllocation>,
    r6: GcRef<TrackedAllocation>,
    r7: GcRef<TrackedAllocation>,
    r8: GcRef<TrackedAllocation>,
    r9: GcRef<TrackedAllocation>,
    r10: GcRef<TrackedAllocation>,
}

#[test]
#[serial]
fn ten_allocations() {
    let tracker = Rc::new(Cell::new(0));
    assert_eq!(tracker.as_ref().get(), 0);

    let refs = TenRefs{
        r1: TrackedAllocation::new(tracker.clone()),
        r2: TrackedAllocation::new(tracker.clone()),
        r3: TrackedAllocation::new(tracker.clone()),
        r4: TrackedAllocation::new(tracker.clone()),
        r5: TrackedAllocation::new(tracker.clone()),
        r6: TrackedAllocation::new(tracker.clone()),
        r7: TrackedAllocation::new(tracker.clone()),
        r8: TrackedAllocation::new(tracker.clone()),
        r9: TrackedAllocation::new(tracker.clone()),
        r10: TrackedAllocation::new(tracker.clone()),
    };

    assert_eq!(tracker.as_ref().get(), 10);
}
