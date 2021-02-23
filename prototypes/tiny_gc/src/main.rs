use bronze::*;
use bronze_derive::*;

trait ATrait {
    fn doit(&self);
}

#[derive(Trace)]
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
    println!("root chain at start of alloc_one_num:");
    print_root_chain();
    let _num_gc_ref_1 = Gc::new(42);

    println!("root chain before alloc_one_num returns:");
    print_root_chain();
    // If I don't collect here, is the shadow stack OK after I return?
    // No, it's still not.
}


fn main() {
    // let a = Gc::new(AStruct {data: 42});
    // let b = Gc::new(AStruct {data: 42});
    println!("initial root chain:");
    print_root_chain();

    alloc_one_num();

    println!("root chain after alloc_one_num returns:");
    print_root_chain();

    force_collect();  // Should traverse the map but not actually collect anything.
}
