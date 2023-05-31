use bronze_gc::*;
use bronze_derive::*;

use std::cell::Cell;
use std::rc::Rc;


trait ATrait {
    fn doit(&self);
}

#[derive(Trace, Finalize)]
struct AStruct {
    data: i32,
}

impl ATrait for AStruct {
    fn doit(&self) {
        println!("Hello, world!");
    }
}

fn alloc_one_num() {
    println!("alloc_one_num");
    let _num_gc_ref_1 = Gc::new(42);
    // If I don't collect here, is the shadow stack OK after I return?
    // No, it's still not.
}



#[derive(Trace, Finalize)]
struct ContainsGc {
    data: bronze_gc::GcRef<u32>,
}

fn use_contains_gc() {
    let c = ContainsGc {data: Gc::new (42)};
}

// struct GcRef {
//     // This is not a Bronze GcRef! We should not erroneously detect it.
// }

fn use_bogus_ref() {
    println!("bogus Gc ref should NOT appear in list below:");
    let _bogus = GcRef {};
}

fn use_vec() {
    println!("should find vec in root chain:");
    let mut v = Vec::new();
    v.push(Gc::new(42));
    #[cfg(feature="enable_garbage_collection")]
    force_collect();  // Should NOT collect the vec.

}

fn use_struct() {
    let _a = Gc::new(AStruct {data: 42});
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

#[derive(Trace, Finalize)]
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

fn ten_allocations() {
    let tracker = Rc::new(Cell::new(0));
    assert_eq!(tracker.as_ref().get(), 0);
    println!("ten_allocations");
    print_root_chain();


    let _x = TrackedAllocation::new(tracker.clone());
    println!("after _x");
    print_root_chain();

    let _y = TrackedAllocation::new(tracker.clone());
    println!("after _y");
    print_root_chain();
 

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
    force_collect();
    assert_eq!(tracker.as_ref().get(), 0);
}

fn main() {
    // println!("initial root chain:");
    // print_root_chain();

    // alloc_one_num();

    // println!("root chain after alloc_one_num returns:");
    // print_root_chain();

    // force_collect();  // Should traverse the map but not actually collect anything.

    // println!("testing useContainsGc");
    // use_contains_gc();
    // force_collect();  // Should collect the old root.

    // use_bogus_ref();

    // use_vec();
    // force_collect();  // Should collect the vec.

    use_struct();
    #[cfg(feature="enable_garbage_collection")]
    force_collect(); // Should collect the struct.

}
