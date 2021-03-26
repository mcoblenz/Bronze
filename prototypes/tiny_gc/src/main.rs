use bronze_gc::*;
use bronze_derive::*;

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

struct GcRef {
    // This is not a Bronze GcRef! We should not erroneously detect it.
}

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
