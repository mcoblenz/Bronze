// Bronze is based in part on code from https://github.com/withoutboats/shifgrethor
// as well as code from https://github.com/Manishearth/rust-gc

// The code pertaining to automatic tracking of borrowed references is from the Rust standard library, which is licensed under the MIT license (https://opensource.org/licenses/MIT).



#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(extern_types)]
#![feature(bronze_gc)]

#[cfg(feature="enable_garbage_collection")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


use std::{marker::PhantomData};
use std::ptr::NonNull;
use std::cell::{Cell, RefCell};
use std::boxed::Box;
use std::mem;
use core::ops::CoerceUnsized;
use std::marker::Unsize;
use std::ops::{Deref, DerefMut};

#[cfg(feature="enable_garbage_collection")]
use std::include;

use std::custom_trace;

mod trace;

//Re-export Finalize and GcTrace.
pub use std::gc::{Finalize, GcTrace};



const INITIAL_THRESHOLD: usize = 100;

// after collection we want the the ratio of used/total to be no
// greater than this (the threshold grows exponentially, to avoid
// quadratic behavior when the heap is growing linearly with the
// number of `new` calls):
const USED_SPACE_RATIO: f64 = 0.7;

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

// Taken from the standard library (core::cell.rs)
type BorrowFlag = isize;
const UNUSED: BorrowFlag = 0;

#[inline(always)]
fn is_writing(x: BorrowFlag) -> bool {
    x < UNUSED
}

#[inline(always)]
fn is_reading(x: BorrowFlag) -> bool {
    x > UNUSED
}

// These go on the Gc heap and hold the actual Gc data.
// GcBoxHeader must occur FIRST so that the GC runtime can find it.
// T is not declared to be GcTrace here so the coercions work later,
// but in fact, there is no way to construct a GcBox with a type that isn't GcTrace.
pub struct GcBox<T: ?Sized + 'static> {
    header: GcBoxHeader,
    borrow_flag: Cell<BorrowFlag>,
    data: T,
}

// GcNullableBox has a storage cost relative to GcBox, so I
// don't want to just replace GcBox with GcNullableBox.
// GcBoxHeader must occur FIRST so that the GC runtime can find it.
pub struct GcNullableBox<T: GcTrace + ?Sized + 'static> {
    header: GcBoxHeader,
    borrow_flag: Cell<BorrowFlag>,
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
        // Do not call finalize_glue here because the generated drop() already calls finalize.
        assert!(self.borrow_flag.get() == UNUSED);
        println!("deallocating GcBox {:p}", self);
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

impl<T: GcTrace> GcRef<T> {
    pub fn new(b: T) -> GcRef<T> {
        let nonnull_ptr = GC_STATE.with(|st| {
            let mut st = st.borrow_mut();

            // Collect if needed. Strategy from Manishearth.

            #[cfg(feature="enable_garbage_collection")]
            if st.bytes_allocated > st.threshold {
                println!("heap getting too full. Automatic garbage collection triggered.");
                collect_garbage(&mut st);

                if st.bytes_allocated as f64 > st.threshold as f64 * USED_SPACE_RATIO {
                    // we didn't collect enough, so increase the
                    // threshold for next time, to avoid thrashing the
                    // collector too much/behaving quadratically.
                    st.threshold = (st.bytes_allocated as f64 / USED_SPACE_RATIO) as usize
                }
            }



            let vtable = extract_vtable(&b);

            let header = GcBoxHeader {
                marked: Cell::new(false),
                vtable: vtable,
                next: st.boxes_start.take()
            };
            let bx_ptr = Box::into_raw(Box::new(GcBox {header, borrow_flag: Cell::new(UNUSED), data: b}));
            let nonnull_ptr = unsafe {NonNull::new_unchecked(bx_ptr)};
            st.boxes_start = Some(nonnull_ptr);
            st.bytes_allocated += mem::size_of::<GcBox<T>>();

            nonnull_ptr
        });

        println!("allocated box {:p}", nonnull_ptr);
        GcBox::ref_from_ptr(nonnull_ptr)
    }
}

// Like Ref.
pub struct GcBorrow<'b, T: GcTrace + ?Sized + 'b> {
    value: &'b T,
    borrow_cell: &'b Cell<BorrowFlag>,
}

impl<'b, T: GcTrace + ?Sized + 'b> GcBorrow<'b, T> {
    fn new(value: &'b T, borrow_cell: &'b Cell<BorrowFlag>) -> Option<GcBorrow<'b, T>> {
        println!("making a new borrow.");

        // Implementation from core::cell::BorrowRef.
        let b = borrow_cell.get().wrapping_add(1);
        if !is_reading(b) {
            // Incrementing borrow can result in a non-reading value (<= 0) in these cases:
            // 1. It was < 0, i.e. there are writing borrows, so we can't allow a read borrow
            //    due to Rust's reference aliasing rules
            // 2. It was isize::MAX (the max amount of reading borrows) and it overflowed
            //    into isize::MIN (the max amount of writing borrows) so we can't allow
            //    an additional read borrow because isize can't represent so many read borrows
            //    (this can only happen if you mem::forget more than a small constant amount of
            //    `Ref`s, which is not good practice)
            None
        } else {
            // Incrementing borrow can result in a reading value (> 0) in these cases:
            // 1. It was = 0, i.e. it wasn't borrowed, and we are taking the first read borrow
            // 2. It was > 0 and < isize::MAX, i.e. there were read borrows, and isize
            //    is large enough to represent having one more read borrow
            borrow_cell.set(b);
            // println!("making a new borrow. New borrow count is {}", b);
            Some(GcBorrow {value: value, borrow_cell: &borrow_cell})
        }
    }
}

impl<T: GcTrace + ?Sized> Drop for GcBorrow<'_, T> {
    fn drop(&mut self) {
        let borrow = self.borrow_cell.get();
        debug_assert!(is_reading(borrow));
        self.borrow_cell.set(borrow - 1);
        // println!("dropping a borrow. New borrow count is {}", borrow - 1);
    }
}

impl<T: GcTrace + ?Sized> Deref for GcBorrow<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

// Like RefMut.
pub struct GcBorrowMut<'b, T: GcTrace + ?Sized + 'b> {
    value: &'b mut T,
    borrow_cell: &'b Cell<BorrowFlag>,
}

impl<'b, T: GcTrace + ?Sized + 'b> GcBorrowMut<'b, T> {
    fn new(value: &'b mut T, borrow_cell: &'b Cell<BorrowFlag>) -> Option<GcBorrowMut<'b, T>> {
        // println!("making a new mutable borrow.");

        // Implementation from core::cell::BorrowRefMut.
        // NOTE: Unlike BorrowRefMut::clone, new is called to create the initial
        // mutable reference, and so there must currently be no existing
        // references. Thus, while clone increments the mutable refcount, here
        // we explicitly only allow going from UNUSED to UNUSED - 1.
        let borrow = borrow_cell.get();
        match borrow {
            UNUSED => {
                borrow_cell.set(UNUSED - 1);
                Some(GcBorrowMut {value: value, borrow_cell: borrow_cell})
            }
            _ => None,
        }
    }
}

impl<T: GcTrace + ?Sized> Drop for GcBorrowMut<'_, T> {
    fn drop(&mut self) {
        let borrow = self.borrow_cell.get();
        debug_assert!(is_writing(borrow));
        self.borrow_cell.set(borrow + 1);
        // println!("dropping a borrow. New borrow count is {}", borrow + 1);
    }
}

impl<T: GcTrace + ?Sized> Deref for GcBorrowMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<T: GcTrace + ?Sized> DerefMut for GcBorrowMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T: GcTrace + ?Sized> GcRef<T> {
    // "as_ref" is used to obtain a reference to the underlying data.
    fn as_ref<'a>(&'a self) -> &'a T {
        unsafe {
            let gc_box = self.obj_ref.as_ref();
            gc_box.as_ref()
        }
    }

    fn as_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe {
            let gc_box = self.obj_ref.as_mut();
            gc_box.as_mut()
        }
    }

    fn borrow_flag<'a>(&'a self) -> &'a Cell<BorrowFlag> {
        unsafe {
            let gc_box = self.obj_ref.as_ref();
            &gc_box.borrow_flag
        }
    }

    pub(crate) fn as_box<'a> (&'a self) -> &GcBox<T> {
        unsafe {
            self.obj_ref.as_ref()
        }
    }

    pub(crate) fn as_mut_box<'a> (&'a mut self) -> &mut GcBox<T> {
        unsafe {
            self.obj_ref.as_mut()
        }
    }

    pub fn borrow(&self) -> GcBorrow<'_, T> {
        let borrow_flag = &self.as_box().borrow_flag;
        match GcBorrow::new(self.as_ref(), borrow_flag) {
            Some(b) => b,
            None => panic!("GC object is already mutably borrowed")
        }
    }

    pub fn borrow_mut(&mut self) -> GcBorrowMut<'_, T> {
        let gc_box = unsafe { self.obj_ref.as_ref() };
        match GcBorrowMut::new(self.as_mut(), &gc_box.borrow_flag) {
            Some(borrow) => borrow,
            None => panic!("GC object is already mutably borrowed")
        }
    }
}

impl<T: GcTrace + ?Sized> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        GcRef {obj_ref: self.obj_ref}
    }
}

impl<T: GcTrace + ?Sized + 'static> Copy for GcRef<T> {}

impl<T: GcTrace + ?Sized> Finalize for GcRef<T> {}
unsafe impl<T: GcTrace + ?Sized> GcTrace for GcRef<T> {
    custom_trace!(this, {
        mark(this.as_ref());
    });
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

impl<T: GcTrace + ?Sized + std::fmt::Debug> std::fmt::Debug for GcRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {(self.obj_ref).as_ref().data.fmt(f)}
    }
}

//===================== GcNullableRef =================
// GcNullableRef can't be defined as GcRef<Option<T>> because Option doesn't support unsized types.

// GcRef represents a reference WITHIN the GC heap.
#[derive(std::cmp::Eq)]
pub struct GcNullableRef<T: GcTrace + ?Sized + 'static> {
    obj_ref: NonNull<GcNullableBox<T>>,
}

impl<T: GcTrace + ?Sized> GcNullableRef<T> {
    // "as_ref" is used to obtain a reference to the underlying data.
    fn as_ref<'a>(&'a self) -> &'a T {
        unsafe {
            let gc_box = self.obj_ref.as_ref();
            gc_box.as_ref()
        }
    }

    fn as_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe {
            let gc_box = self.obj_ref.as_mut();
            gc_box.as_mut()
        }
    }
    pub fn borrow(&self) -> GcBorrow<'_, T> {
        let gc_box = unsafe { self.obj_ref.as_ref() };
        match GcBorrow::new(self.as_ref(), &gc_box.borrow_flag) {
            Some(b) => b,
            None => panic!("GC object is already mutably borrowed")
        }
    }

    pub fn borrow_mut(&mut self) -> GcBorrowMut<'_, T> {
        let gc_box = unsafe { self.obj_ref.as_ref() };
        match GcBorrowMut::new(self.as_mut(), &gc_box.borrow_flag) {
            Some(b) => b,
            None => panic!("GC object is already mutably borrowed")
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

impl<T: GcTrace + ?Sized> Finalize for GcNullableRef<T> {}
unsafe impl<T: GcTrace + ?Sized> GcTrace for GcNullableRef<T> {
    custom_trace!(this, {
        mark(this.as_ref());
    });
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

// For testing purposes
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
        GcRef::new(b)
    }


    pub fn new_nullable(b: T) -> GcNullableRef<T> {
        let nonnull_ptr = GC_STATE.with(|st| {
            let mut st = st.borrow_mut();

            // TODO: collect if needed
            let vtable = extract_vtable(&b);

            let header = GcBoxHeader {
                marked: Cell::new(false),
                vtable: vtable,
                next: st.boxes_start.take()
            };
            let bx_ptr = Box::into_raw(Box::new(GcNullableBox {is_null: false, header, borrow_flag: Cell::new(UNUSED), data: b}));
            let nonnull_ptr = unsafe {NonNull::new_unchecked(bx_ptr)};
            st.nonnull_boxes_start = Some(nonnull_ptr);
            st.bytes_allocated += mem::size_of::<GcBox<T>>();

            nonnull_ptr
        });

        GcNullableBox::ref_from_ptr(nonnull_ptr)
    }

}


#[cfg(feature="enable_garbage_collection")]
pub fn force_collect() {
    GC_STATE.with(|st| {
        let mut st = st.borrow_mut();
        collect_garbage(&mut st);
    });
}

#[cfg(feature="enable_garbage_collection")]
fn gc_root_chain() -> *const LLVMStackEntry {
    unsafe {
        get_llvm_gc_root_chain()
    }
}

#[cfg(feature="enable_garbage_collection")]
fn iterate_roots<F>(f: F) 
    where F: Fn(*mut c_void, *const u8) {
    
    unsafe { 
        let mut stack_entry = gc_root_chain();
        while !stack_entry.is_null() {
            let frame_map = (*stack_entry).Map;
            // println!("stack entry {:p}", stack_entry);
            if !frame_map.is_null() {
                // println!("frame map {:p}", frame_map);
                let num_roots = (*frame_map).NumRoots; 
                // println!("{} roots found in this frame map", num_roots);

                let roots = (*stack_entry).Roots.as_slice(num_roots as usize);

                let num_meta = (*frame_map).NumMeta as usize;
                let meta = (*frame_map).Meta.as_slice(num_meta as usize);
                // Metadata is only present for "fat" roots, to indicate that they are actually pointers to real root pointers.                    

                for i in 0..num_roots as usize {
                    let mut root = roots[i];
                    let meta = meta[i];

                    // println!("root {:p} meta: {:?}", root, meta);

                    if !root.is_null() {
                        let meta_ptr = meta as usize;
                        match meta_ptr {
                            2 => {
                                // This is a fat pointer, so there's a layer of indirection there.
                                // Root should be interpreted as a pointer to the real root.
                                let root_ptr = root as *const *mut c_void;
                                root = *root_ptr;
                            },
                            1 => (), // No metadata here; this is a regular root (just a GcRef)
                            _ => panic!("Invalid metadata found for root {:p}. meta: {:?}", root, meta)
                        }
                        f(root, meta as *const u8);
                    }
                }
            }
            stack_entry = (*stack_entry).Next;
        }
    }
}
#[cfg(feature="enable_garbage_collection")]
fn collect_garbage(st: &mut GcState) {
    fn mark() {
        println!("marking all roots");
        iterate_roots(|root, _meta| {
            unsafe {
                let root_as_gcbox: *mut GcBox<Data> = mem::transmute(root);
                (*root_as_gcbox).trace_inner();
            }
        });
    }

    fn sweep(state: &mut GcState) {
        let mut a_box = state.boxes_start;
    
        // println!("sweeping");

        if a_box.is_some() {
            let mut prev_box = a_box.expect("cannot have empty Option here").as_ptr();

            while a_box.is_some() {
                let a_box_nonnull_ref = a_box.expect("cannot have empty Option here");
                let gc_ref = GcBox::ref_from_ptr(a_box_nonnull_ref);
                let gc_box_ref = gc_ref.as_box();
                // println!("sweeping {:p}", gc_box_ref);

                unsafe {
                    if !gc_box_ref.header.marked.get() {
                        // println!("Should collect {:p}", gc_box_ref);
                        let next_box_ref = a_box_nonnull_ref.as_ref().header.next;

                        if prev_box == state.boxes_start.expect("must have at least one box").as_ptr() {
                            // Deleting the first node.
                            state.boxes_start = next_box_ref;
                            // println!("deleted the first node")
                        }
                        else {
                            (*prev_box).header.next = next_box_ref;
                            // println!("deleted a node in the middle");
                        }

                        let bytes_freed = mem::size_of_val::<GcBox<_>>(gc_box_ref);
                        println!("freed {} bytes", bytes_freed);
                        state.bytes_allocated -= bytes_freed;

                        // Inflating the box will result in Rust freeing it when _inflated_box goes out of scope.
                        // Freeing the box will call drop(), which will call finalize_glue(), so we don't call finalize separately here.
                        let _inflated_box = Box::from_raw(a_box_nonnull_ref.as_ptr());
                    }
                    else {
                        // println!("Not collecting {:p} (it is marked)", gc_box_ref);
                    }
                    prev_box = a_box_nonnull_ref.as_ptr();
                    a_box = a_box_nonnull_ref.as_ref().header.next;
                }
            }
        }
    }


    fn clear_marks(state: &mut GcState) {
        let mut a_box = state.boxes_start;
        while a_box.is_some() {
            let ptr = a_box.expect("cannot have empty Option here");
            
            unsafe {
                let gc_ref = GcBox::ref_from_ptr(ptr);
                let gc_box = gc_ref.as_box();
                gc_box.header.marked.set(false);
                // println!("Cleared mark on {:p}", gc_box);

                a_box = ptr.as_ref().header.next;
            }
        }
    }

    mark();
    sweep(st);
    clear_marks(st);
}

#[cfg(feature="enable_garbage_collection")]
pub fn print_root_chain() {
    println!("Root chain: ");
    iterate_roots(|root, meta| {
        println!("    root {:p} meta: {:?}", root, meta);
    });
}