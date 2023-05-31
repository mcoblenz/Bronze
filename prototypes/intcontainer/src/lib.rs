use bronze_gc::GcRef;
use bronze_derive::*;

#[derive(Trace, Finalize)]
pub struct IntContainer {
    n: i32,
}

pub fn set(mut c: GcRef<IntContainer>, n: i32) {
    c.n = n;
}

pub fn test() {
    let c1 = GcRef::new(IntContainer{n: 42});
    let c2 = c1; 
    // Now c1 and c2 both reference the same object.
    
    set(c2, 42);
    set(c1, 43);
    // Now they both reference an object with value 43.
}