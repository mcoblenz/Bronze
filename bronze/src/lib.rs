#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![feature(coerce_unsized)]
#![feature(unsize)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


use std::marker::PhantomData;
use std::ptr::NonNull;
use std::cell::RefCell;
use multiset::HashMultiSet;
use std::boxed::Box;
use std::mem;
use core::ops::{Deref};

use core::ops::CoerceUnsized;
use std::marker::Unsize;

use std::include;



pub trait GcTrace {

}

impl GcTrace for u32 {}

/*
* I want the following kinds of references to Gc objects:
* 1. References IN the Gc heap (GcRef)
* 2. References in the roots collection (GcRoot)
* 3. References in the Rust heap (GcHandle)
*/

// The copypasta between nullable and regular types should be removed somehow.


// Based on rust-gc by Manishearth.
// These go on the Gc heap and hold the actual Gc data.
pub struct GcBox<T: GcTrace + ?Sized + 'static> {
    data: T,
}

// GcNullableBox has a storage cost relative to GcBox, so I
// don't want to just replace GcBox with GcNullableBox.
pub struct GcNullableBox<T: GcTrace + ?Sized + 'static> {
    is_null: bool,
    data: T,
}

impl<T: GcTrace + 'static> GcNullableBox<T> {
    pub fn take(&mut self) -> Option<T> {
        if self.is_null {
            None
        }
        else {
            self.is_null = true; 
            unsafe {
                // Can't move even from behind a raw pointer,
                // so I appear to be stuck with transmute_copy here
                // (to avoid requiring T to implement the Default trait).
                let result: T = mem::transmute_copy(&self.data);
                Some(result)
            }
        }
    }
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

    pub fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: GcTrace + ?Sized> Drop for GcBox<T> {
    fn drop(&mut self) {
        println!("deallocating a GcBox.");
    }
}

impl<T: GcTrace> GcNullableBox<T> {
    pub fn new(data: T) -> NonNull<Self> {
        let bx = Box::new(GcNullableBox {is_null: false, data});
        let bx_ptr = Box::into_raw(bx);

        unsafe {NonNull::new_unchecked(bx_ptr)}
    }
}

impl<T: GcTrace + ?Sized> GcNullableBox<T> {
    pub fn new_ref(&mut self) -> GcNullableRef<T> {
        let ptr: *mut Self = self;

        GcNullableRef {obj_ref: unsafe {NonNull::new_unchecked(ptr)}}
    }

    pub fn ref_from_ptr(ptr: NonNull<Self>) -> GcNullableRef<T> {
        GcNullableRef {obj_ref: ptr}
    }

    pub fn as_ref(&self) -> &T {
        &self.data
    }

    pub fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

//================= GcRef =================
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

    pub fn as_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe {
            let gc_box = self.obj_ref.as_mut();
            gc_box.as_mut()
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

// See https://github.com/rust-lang/rust/issues/27732
impl<T, U> CoerceUnsized<GcRef<U>> for GcRef<T> 
    where T: Unsize<U> + GcTrace + ?Sized,
    U: GcTrace + ?Sized 
    {}

//===================== GcNullableRef =================

// GcRef represents a reference WITHIN the GC heap.
#[derive(std::cmp::Eq)]
pub struct GcNullableRef<T: GcTrace + ?Sized + 'static> {
    obj_ref: NonNull<GcNullableBox<T>>,
}

impl<T: GcTrace + ?Sized> GcNullableRef<T> {
    // "as_ref" is used to obtain a reference to the underlying data.
    pub fn as_ref<'a>(&'a self) -> &'a T {
        unsafe {
            let gc_box = self.obj_ref.as_ref();
            gc_box.as_ref()
        }
    }

    pub fn as_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe {
            let gc_box = self.obj_ref.as_mut();
            gc_box.as_mut()
        }
    }
}

impl<T: GcTrace + ?Sized> Clone for GcNullableRef<T> {
    fn clone(&self) -> Self {
        GcNullableRef {obj_ref: self.obj_ref}
    }
}

impl<T: GcTrace> GcNullableRef<T> {
    // Take only works once! Returns None if the value was already taken.
    pub fn remove(&mut self) -> Option<T> {
        unsafe {
            let gc_box = self.obj_ref.as_mut();
            gc_box.take()
        }
    }
}

impl<T: GcTrace + ?Sized + 'static> Copy for GcNullableRef<T> {}

impl<T: GcTrace + ?Sized> GcTrace for GcNullableRef<T> {
    // TODO
}


impl<T: ?Sized + GcTrace> PartialEq for GcNullableRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.obj_ref == other.obj_ref
    }
}

impl<T: ?Sized + GcTrace> std::hash::Hash for GcNullableRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj_ref.hash(state);
    }
}

impl<T: GcTrace + ?Sized + std::fmt::Display> std::fmt::Display for GcNullableRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {(self.obj_ref).as_ref().data.fmt(f)}
    }
}

//===================== GcUntypedRoot =================


#[derive(Hash, Debug, Clone)]
enum GcMaybeNullableRoot {
    BoxRef(NonNull<GcBox<dyn GcTrace>>),
    NullableBoxRef(NonNull<GcNullableBox<dyn GcTrace>>),
}

impl std::cmp::PartialEq for GcMaybeNullableRoot {
    fn eq(&self, other: &Self) -> bool {
        match self {
            GcMaybeNullableRoot::BoxRef(ptr1) => match other {
                GcMaybeNullableRoot::BoxRef(ptr2) => ptr1 == ptr2,
                _ => false
            }
            GcMaybeNullableRoot::NullableBoxRef(ptr1) => match other {
                GcMaybeNullableRoot::NullableBoxRef(ptr2) => ptr1 == ptr2,
                _ => false
            }
        }
    }
}

impl Copy for GcMaybeNullableRoot {}

#[derive(Hash, Debug)]
pub struct GcUntypedRoot {
    gc_ref: GcMaybeNullableRoot,
}

impl GcUntypedRoot {
    pub fn new(gc_ref: NonNull<GcBox<dyn GcTrace>>) -> Self {
        GcUntypedRoot{gc_ref: GcMaybeNullableRoot::BoxRef (gc_ref)}
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

impl Copy for GcUntypedRoot {}

// GcHandle is Move, not Copy.
// GcHandle represents a reference from linear Rust code to the GC heap.
#[derive(std::cmp::Eq)]
pub struct GcHandle<T: GcTrace + 'static + ?Sized> {
    gc_ref: GcRef<T>,
    untyped_root: GcUntypedRoot,
}

//If you have a GcHandle, you can borrow a 
// mutable reference to the underlying data, 
// but that consumes the GcHandle 
// (so that you can't borrow a second mutable reference).
impl<T: GcTrace> GcHandle<T> {
    pub fn new(gc_ref: GcRef<T>) -> GcHandle<T> {
        let untyped_root = GcUntypedRoot::new(gc_ref.obj_ref);

        ROOTS.with(|roots| {
            (*roots.borrow_mut()).add_root(untyped_root);
        });

        GcHandle {gc_ref, untyped_root}
    }

    pub fn gc_ref(&self) -> GcRef<T> {
        self.gc_ref
    }
}

impl<T: GcTrace + ?Sized> GcHandle<T> {
    // "as_ref" is used to obtain a reference to the underlying data.
    pub fn as_ref<'a>(&'a self) -> &'a T {
        unsafe {
            self.gc_ref.as_ref()
        }
    }

    pub fn as_mut<'a>(&'a mut self) -> &'a mut T {
        self.gc_ref.as_mut()
    }
}

impl<T: GcTrace> Deref for GcHandle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.gc_ref.as_ref()
    }
}

impl<T: GcTrace + ?Sized> Drop for GcHandle<T> {
    fn drop(&mut self) { 
        ROOTS.with(|roots| {
            // let trace_ref = self.gc_ref as GcRef<dyn GcTrace>;


            // let obj_ref: NonNull<GcBox<T>> = self.gc_ref.obj_ref;
            // let trace_ref = obj_ref as NonNull<GcBox<dyn GcTrace>>;
            // let untyped_root = GcUntypedRoot::new(trace_ref.obj_ref);
            roots.borrow_mut().drop_root(self.untyped_root);
        });
     }
}

impl<T: ?Sized + GcTrace> PartialEq for GcHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.gc_ref == other.gc_ref
    }
}

impl<T, U> CoerceUnsized<GcHandle<U>> for GcHandle<T> 
    where T: Unsize<U> + GcTrace + ?Sized,
    U: GcTrace + ?Sized 
    {}

// Master list of all roots, for use in doing tracing.
pub struct GcRoots {
    roots: HashMultiSet<GcUntypedRoot>,
}

impl GcRoots {
    fn new() -> Self {
        GcRoots {roots: HashMultiSet::new()}
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
        unsafe {
            bronze_init(); // TODO: move this so it only happens once
            trace_roots();
        }

        let gc_box = GcBox::new(b);
        GcBox::ref_from_ptr(gc_box)
    }

    pub fn new_handle(b: T) -> GcHandle<T> {
        let gcRef = Self::new(b);
        GcHandle::new(gcRef)
    }

    pub fn new_nullable(b: T) -> GcNullableRef<T> {
        let gc_box = GcNullableBox::new(b);
        GcNullableBox::ref_from_ptr(gc_box)
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


    pub trait Shape {}
    struct Square {}
    impl Shape for Square {}
    impl GcTrace for Square {}
    impl GcTrace for dyn Shape {}

    fn take_shape(_shape: GcRef<dyn Shape>) {}

    fn take_shape_box(_shape: Box<dyn Shape>) {}

    #[test]
    fn unsize_test() {
        let sq = Square{};

        let gc_square = Gc::new(sq);
        take_shape(gc_square);
    }

    #[test]
    fn box_test() {
        let sq = Square{};
        let b = Box::new(sq);

        take_shape_box(b);
    }

    #[test]
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
            assert_eq!(roots_len(), 1);
            let num_handle2 = GcHandle::new(num_gc_ref);
            assert_eq!(roots_len(), 2);
        }
        assert_eq!(roots_len(), 0);
    }
}



fn trace_roots() {
    unsafe {
        let mut stack_entry = llvm_gc_root_chain_bronze_ref;
        while !stack_entry.is_null() {
            let frame_map = (*stack_entry).Map;
            println!("stack entry");
            if !frame_map.is_null() {
                let num_roots = (*frame_map).NumRoots; 
                println!("{} roots found in this frame map", num_roots);
                let roots = (*stack_entry).Roots.as_slice(num_roots as usize);

                let num_meta = (*frame_map).NumMeta;
                let meta = (*frame_map).Meta.as_slice(num_meta as usize);

                assert!(num_meta == num_roots, "Every root must have metadata; otherwise we won't know how to trace some roots.");

                for i in 0..num_roots as usize {
                    let root = roots[i];
                    let meta = meta[i];

                    println!("root {:p} meta: {:?}", root, meta);

                }
            }
            stack_entry = (*stack_entry).Next;
        }
    }
}
