use std::marker::PhantomData;
use std::ptr::NonNull;
use std::cell::RefCell;
use std::collections::HashSet;
use std::boxed::Box;
use core::ops::Deref;

pub trait GcTrace {

}

impl GcTrace for u32 {}

/*
* I want the following kinds of references to Gc objects:
* 1. References IN the Gc heap (GcRef)
* 2. References in the roots collection (GcRoot)
* 3. References in the Rust heap (GcHandle)
*/


// Based on rust-gc by Manishearth.
// These go on the Gc heap and hold the actual Gc data.
pub struct GcBox<T: GcTrace + ?Sized + 'static> { // TODO: make T 'static?
    data: T,
}

impl<T: GcTrace> GcBox<T> {
    pub fn new(data: T) -> NonNull<Self> {
        let bx = Box::new(GcBox {data});
        let bx_ptr = Box::into_raw(bx);

        unsafe {NonNull::new_unchecked(bx_ptr)}
    }
}

impl<T: GcTrace + ?Sized> GcBox<T> {
    pub fn new_ref(&mut self) -> GcRef<T> {
        let ptr: *mut Self = self;

        GcRef {obj_ref: unsafe {NonNull::new_unchecked(ptr)}}
    }

    pub fn ref_from_ptr(ptr: NonNull<Self>) -> GcRef<T> {
        GcRef {obj_ref: ptr}
    }

    pub fn as_ref(&self) -> &T {
        &self.data
    }

    pub fn as_mut_ref(&mut self) -> &mut T {
        &mut self.data
    }
}

// GcRef represents a reference WITHIN the GC heap.
#[derive(std::cmp::Eq)]
pub struct GcRef<T: GcTrace + ?Sized + 'static> {
    obj_ref: NonNull<GcBox<T>>,
}

impl<T: GcTrace + ?Sized> GcRef<T> {
    // "as_ref" is used to obtain a reference to the underlying data.
    pub fn as_ref<'a>(&'a self) -> &'a T {
        unsafe {
            let gc_box = self.obj_ref.as_ref();
            gc_box.as_ref()
        }
    }

    pub fn as_mut_ref<'a>(&'a mut self) -> &'a mut T {
        unsafe {
            let gc_box = self.obj_ref.as_mut();
            gc_box.as_mut_ref()
        }
    }
}

impl<T: GcTrace + ?Sized> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        GcRef {obj_ref: self.obj_ref}
    }
}

impl<T: GcTrace + ?Sized + 'static> Copy for GcRef<T> {}

impl<T: GcTrace + ?Sized> GcTrace for GcRef<T> {
    // TODO
}


impl<T: ?Sized + GcTrace> PartialEq for GcRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.obj_ref == other.obj_ref
    }
}

impl<T: ?Sized + GcTrace> std::hash::Hash for GcRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj_ref.hash(state);
    }
}

impl<T: GcTrace + ?Sized + std::fmt::Display> std::fmt::Display for GcRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {(self.obj_ref).as_ref().data.fmt(f)}
    }
}

#[derive(Hash, Debug)]
struct GcUntypedRoot {
    gc_ref: NonNull<GcBox<dyn GcTrace>>,
}

impl GcUntypedRoot {
    pub fn new(gc_ref: NonNull<GcBox<dyn GcTrace>>) -> Self {
        // Transmute here lets us ignore the lifetime of T.
        GcUntypedRoot {gc_ref}
    }
}

impl PartialEq for GcUntypedRoot {
    fn eq(&self, other: &Self) -> bool {
        self.gc_ref == other.gc_ref
    }
}

impl Eq for GcUntypedRoot {}

// Can't automatically derive Clone due to dyn GcTrace type.
impl Clone for GcUntypedRoot {
    fn clone(&self) -> Self {
        GcUntypedRoot {gc_ref: self.gc_ref.clone()}
    }
}

// GcHandle is Move, not Copy.
// GcHandle represents a reference from linear Rust code to the GC heap.
pub struct GcHandle<T: GcTrace + 'static> {
    gc_ref: GcRef<T>,
}

//If you have a GcHandle, you can borrow a 
// mutable reference to the underlying data, 
// but that consumes the GcHandle 
// (so that you can't borrow a second mutable reference).
impl<T: GcTrace> GcHandle<T> {
    pub fn new(gc_ref: GcRef<T>) -> GcHandle<T> {
        ROOTS.with(|roots| {
            let untyped_root = GcUntypedRoot::new(gc_ref.obj_ref);
            (*roots.borrow_mut()).add_root(untyped_root);
        });

        GcHandle {gc_ref}
    }
}

impl<T: GcTrace> Deref for GcHandle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.gc_ref.as_ref()
    }
}

impl<T: GcTrace + Sized> Drop for GcHandle<T> {
    fn drop(&mut self) { 
        ROOTS.with(|roots| {
            let untyped_root = GcUntypedRoot::new(self.gc_ref.obj_ref);
            roots.borrow_mut().drop_root(untyped_root);
        });
     }
}

// Master list of all roots, for use in doing tracing.
pub struct GcRoots {
    roots: HashSet<GcUntypedRoot>,
}

impl GcRoots {
    fn new() -> Self {
        GcRoots {roots: HashSet::new()}
    }

    fn add_root(&mut self, root: GcUntypedRoot) {
        println!("Adding root {:?} to roots list.", root);
        self.roots.insert(root);
    }

    fn drop_root(&mut self, root: GcUntypedRoot) {
        self.roots.remove(&root);
        println!("Dropped root {:?} from roots list.", root)
    }

    fn len(&self) -> usize {
        self.roots.len()
    }

    // TODO
    fn trace() {
        todo!();
    }
}

// Each thread has its own list of roots, since GC references are neither Send nor Sync.
thread_local! {
    pub static ROOTS: RefCell<GcRoots> = RefCell::new(GcRoots::new());
}

pub fn roots_len() -> usize {
    ROOTS.with(|roots| {
        return (*roots.borrow()).len()
    })
}

// This is for prototyping only.
pub struct Gc<T: ?Sized> {
    phantom: PhantomData<T>,
}

impl<T: GcTrace> Gc<T> {
    pub fn new(b: T) -> GcRef<T> {
        let gc_box = GcBox::new(b);
        GcBox::ref_from_ptr(gc_box)
    }
}

// TODO
impl<T: GcTrace + ?Sized> GcTrace for Box<T> {}


#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn new_ref() {
        let num_ref = Gc::new(42);
        let ref_alias = num_ref;
        let gc_num_ref = ref_alias.as_ref();

        // GcRef is Copy, so this is fine.
        let gc_num_ref2 = num_ref.as_ref();

        assert_eq!(*gc_num_ref, 42);
        assert_eq!(*gc_num_ref2, 42);

    }

    #[test]
    fn handles() {
        let num_gc_ref = Gc::new(42);
        let num_handle = GcHandle::new(num_gc_ref);
        assert_eq!(*num_handle, 42);

        let moved_handle = num_handle;

        // Should error because num_handle has been consumed.
        //assert_eq!(*num_handle, 42);
       
        assert_eq!(*moved_handle, 42);
    }

    fn roots() {
        assert_eq!(roots_len(), 0);
        let num_gc_ref = Gc::new(42);
        {
            let num_handle = GcHandle::new(num_gc_ref);
            assert_eq!(roots_len(), 1);
            let _moved_handle = num_handle;
            assert_eq!(roots_len(), 1);
        }

        // Handles are now out of scope, so size should be 0.
        assert_eq!(roots_len(), 0);

        {
            // Two handles for the same object.
            let num_handle = GcHandle::new(num_gc_ref);
            let num_handle2 = GcHandle::new(num_gc_ref);
            assert_eq!(roots_len(), 2);
        }
        assert_eq!(roots_len(), 0);
    }
}
