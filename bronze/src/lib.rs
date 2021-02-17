// Bronze is based in part on code from https://github.com/withoutboats/shifgrethor
// as well as code from https://github.com/Manishearth/rust-gc


#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(extern_types)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


use std::marker::PhantomData;
use std::ptr::NonNull;
use std::cell::{Cell, RefCell};
use mem::{size_of, transmute_copy};
use std::boxed::Box;
use std::mem;
use core::{num, ops::{Deref}};

use core::ops::CoerceUnsized;
use std::marker::Unsize;

use std::include;
use serial_test::serial;

const INITIAL_THRESHOLD: usize = 100;

struct GcState {
    bytes_allocated: usize,
    threshold: usize,
    nonnull_boxes_start: Option<NonNull<GcNullableBox<dyn GcTrace>>>,
    boxes_start: Option<NonNull<GcBox<dyn GcTrace>>>,
}

// https://github.com/rust-lang/rfcs/blob/master/text/1861-extern-types.md
extern {
    pub type Data;
    // TODO: Do I need this Vtable type?
    type Vtable;
}

pub unsafe trait GcTrace {
    unsafe fn trace(&self);
}

unsafe impl GcTrace for u32 {
    unsafe fn trace(&self) {}
}

unsafe impl GcTrace for i32 {
    unsafe fn trace(&self) {}
}

unsafe impl GcTrace for f64 {
    unsafe fn trace(&self) {}
}

unsafe impl<T: GcTrace> GcTrace for Option<T> {
    unsafe fn trace(&self) {
        match self {
            None => (),
            Some(x) => x.trace()
        }
    }
}

unsafe impl<T: GcTrace> GcTrace for Vec<T> {
    unsafe fn trace(&self) {
        for x in self {
            x.trace();
        }
    }
}


/*
* I want the following kinds of references to Gc objects:
* 1. References IN the Gc heap (GcRef)
* 2. References in the roots collection (GcRoot)
* 3. References in the Rust heap (GcHandle)
*/

// The copypasta between nullable and regular types should be removed somehow.


// Based on rust-gc by Manishearth.
struct GcBoxHeader {
    next: Option<NonNull<GcBox<dyn GcTrace>>>,
    vtable: *mut Vtable,

    // TODO: optimize this by moving this bit elsewhere.
    marked: Cell<bool>,
}

// These go on the Gc heap and hold the actual Gc data.
// GcBoxHeader must occur FIRST so that the GC runtime can find it.
// T is not declared to be GcTrace here so the coercions work later,
// but in fact, there is no way to construct a GcBox with a type that isn't GcTrace.
pub struct GcBox<T: ?Sized + 'static> {
    header: GcBoxHeader,
    data: T,
}

// GcNullableBox has a storage cost relative to GcBox, so I
// don't want to just replace GcBox with GcNullableBox.
// GcBoxHeader must occur FIRST so that the GC runtime can find it.
pub struct GcNullableBox<T: GcTrace + ?Sized + 'static> {
    header: GcBoxHeader,
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

impl<T: ?Sized> GcBox<T> {
    pub(crate) unsafe fn trace_inner(&self) {
        let marked = self.header.marked.get();
        if !marked {
            self.header.marked.set(true);
            // println!("marked box {:p}", self);

            let traceable = self.dyn_data();
            traceable.trace();
        }
    }


    fn erased(&self) -> &GcBox<Data> {
        unsafe {
            &*(self as *const GcBox<T> as *const GcBox<Data>)
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    fn dyn_data(&self) -> &dyn GcTrace {
        unsafe {
            let object = Object {
                data: self.erased().data() as *const Data,
                vtable: self.header.vtable,
            };
            mem::transmute::<Object, &dyn GcTrace>(object)
        }
    }
}

impl<T: ?Sized> Drop for GcBox<T> {
    fn drop(&mut self) {
        println!("deallocating a GcBox.");
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

// This corresponds with the memory layout of Rust fat pointers.
#[repr(C)]
struct Object {
    data: *const Data,
    vtable: *mut Vtable,
}

fn extract_vtable<T: GcTrace>(data: &T) -> *mut Vtable {
    unsafe {
        let obj = data as &dyn GcTrace;
        mem::transmute::<&dyn GcTrace, Object>(obj).vtable
    }
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

    pub(crate) fn as_box<'a> (&'a self) -> &GcBox<T> {
        unsafe {
            self.obj_ref.as_ref()
        }
    }
}

impl<T: GcTrace + ?Sized> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        GcRef {obj_ref: self.obj_ref}
    }
}

impl<T: GcTrace + ?Sized + 'static> Copy for GcRef<T> {}

unsafe impl<T: GcTrace + ?Sized> GcTrace for GcRef<T> {
    unsafe fn trace(&self) {
        self.as_box().trace_inner();
    }
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

impl<T: GcTrace + ?Sized + 'static> Copy for GcNullableRef<T> {
}

unsafe impl<T: GcTrace + ?Sized> GcTrace for GcNullableRef<T> {
    unsafe fn trace(&self) {
        // TODO
    }
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


// Each thread has its own list of roots, since GC references are neither Send nor Sync.
thread_local!(static GC_STATE: RefCell<GcState> = RefCell::new(GcState {
    bytes_allocated: 0,
    threshold: INITIAL_THRESHOLD,
    boxes_start: None,
    nonnull_boxes_start: None,
}));

pub fn boxes_len() -> usize {
    let mut num_roots  = 0;
    GC_STATE.with(|st| {
        let state = st.borrow();
        let mut a_box = state.boxes_start;
        while a_box.is_some() {
            let ptr = a_box.expect("cannot have empty Option here");
            num_roots = num_roots + 1;
            unsafe {
                a_box = ptr.as_ref().header.next;
            }
        }
    });

    num_roots
}

// This is for prototyping only.
pub struct Gc<T: ?Sized> {
    phantom: PhantomData<T>,
}

impl<T: GcTrace> Gc<T> {
    pub fn new(b: T) -> GcRef<T> {
        unsafe {
            bronze_init(); // TODO: move this so it only happens once
            mark();
        }

        let nonnull_ptr = GC_STATE.with(|st| {
            let mut st = st.borrow_mut();

            // TODO: collect if needed
            let vtable = extract_vtable(&b);

            let header = GcBoxHeader {
                marked: Cell::new(false),
                vtable: vtable,
                next: st.boxes_start.take()
            };
            let bx_ptr = Box::into_raw(Box::new(GcBox {header, data: b}));
            let nonnull_ptr = unsafe {NonNull::new_unchecked(bx_ptr)};
            st.boxes_start = Some(nonnull_ptr);
            st.bytes_allocated += mem::size_of::<GcBox<T>>();

            nonnull_ptr
        });

        GcBox::ref_from_ptr(nonnull_ptr)
    }


    pub fn new_nullable(b: T) -> GcNullableRef<T> {
        unsafe {
            bronze_init(); // TODO: move this so it only happens once
            mark();
        }

        let nonnull_ptr = GC_STATE.with(|st| {
            let mut st = st.borrow_mut();

            // TODO: collect if needed
            let vtable = extract_vtable(&b);

            let header = GcBoxHeader {
                marked: Cell::new(false),
                vtable: vtable,
                next: st.boxes_start.take()
            };
            let bx_ptr = Box::into_raw(Box::new(GcNullableBox {is_null: false, header, data: b}));
            let nonnull_ptr = unsafe {NonNull::new_unchecked(bx_ptr)};
            st.nonnull_boxes_start = Some(nonnull_ptr);
            st.bytes_allocated += mem::size_of::<GcBox<T>>();

            nonnull_ptr
        });

        GcNullableBox::ref_from_ptr(nonnull_ptr)
    }

}

// TODO
unsafe impl<T: GcTrace + ?Sized> GcTrace for Box<T> {
    unsafe fn trace(&self) {
        // TODO
    }
}


// Tests must be run sequentially because the shadow stack implementation does not support concurrency.
// Unfortunately Rust doesn't support configuring tests to run with only one thread, so we have to use #[serial] on every test!
// https://github.com/rust-lang/rust/issues/43155
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    #[serial]
    fn new_ref() {
        let num_ref = Gc::new(42);
        let ref_alias = num_ref;
        let gc_num_ref = ref_alias.as_ref();

        // GcRef is Copy, so this is fine.
        let gc_num_ref2 = num_ref.as_ref();

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
        println!("first ref: {:p}", _num_gc_ref_1.obj_ref);
        let _num_gc_ref_2 = Gc::new(42);
        assert_eq!(boxes_len(), 2);
        println!("second ref: {:p}", _num_gc_ref_2.obj_ref);
    }

    struct OneRef {
        r: GcRef<i32>,
    }

    unsafe impl GcTrace for OneRef {
        unsafe fn trace(&self) {
            self.r.trace();
        }
    }

    #[test]
    #[serial]
    fn one_ref() {
        assert_eq!(boxes_len(), 0);
        let num_gc_ref_1 = Gc::new(42);
        let oneRef_1 = Gc::new(OneRef{r: num_gc_ref_1});
        let num_gc_ref_2 = Gc::new(42);
        let oneRef_2 = Gc::new(OneRef{r: num_gc_ref_2});
        assert_eq!(boxes_len(), 4);
    }
}


fn mark() {
    println!("mark");
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

                    if !root.is_null() {
                        // Assumes that the header occurs FIRST in the box!
                        let root_as_gcbox: *mut GcBox<Data> = mem::transmute(root);
                        (*root_as_gcbox).trace_inner();
                    }
                }
            }
            stack_entry = (*stack_entry).Next;
        }
    }
}
