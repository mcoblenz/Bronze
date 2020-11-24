use std::rc::Rc;
use std::cell::UnsafeCell;
use std::marker::PhantomData;


trait GCTraceable {

}

/*
* I want the following kinds of references to GC objects:
* 1. References IN the GC heap (GCRef)
* 2. References in the roots collection (GCRoot)
* 3. References in the Rust heap (GCHandle)
*/

// References WITHIN the GC heap.
// The compiler doesn't need to know how big a T is because it's just going to be a reference.
// pub struct GCRef<T: ?Sized> {
//     // No reference counting here; we'll trace instead.
//     obj_ref: *const GCUntypedRoot,
//     phantom: PhantomData<T>,
// }


// For now, let's keep it simple. References WITHIN the GC heap are merely
// unsafe cell references, which leak.
pub struct GCRef<T: ?Sized> {
    obj_ref: Box<T> // TODO: make these unsafe cells?
}

impl<T> GCRef<T> {
    fn new(x: Box<T>) -> Self {
        GCRef {obj_ref: x}
    }
}

impl<T: ?Sized> std::fmt::Display for GCRef<T> 
    where T: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.obj_ref.fmt(f)
    }
}

struct GCUntypedRoot {
    obj: UnsafeCell<dyn GCTraceable>
}

struct GCHandle<T> {
    obj_ref: Rc<GCUntypedRoot>,
    phantom: PhantomData<T>,
}

// Master list of all roots, for use in doing tracing.
struct GCRoots {
    
}


// This is for prototyping only.
pub struct GC<T: ?Sized> {
    phantom: PhantomData<T>,
}

impl<T: ?Sized> GC<T> {
    pub fn new(b: Box<T>) -> GCRef<T> {
       
        GCRef{obj_ref: b}
    }
}




#[cfg(test)]
mod tests {
    #[test]
    fn new_handle() {
        assert_eq!(2 + 2, 4);
    }
}
