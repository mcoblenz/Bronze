# Bronze

## Installation

To use Bronze, you must use the nightly Rust tools. To install them, run `rustup default nightly` on the command line.

## Status

This version of Bronze is similar to the version that was used in the experiment reported in the paper, with the addition of a fix for soundness. IMPORTANT: the actual collector, which was incomplete, is _disabled_. This means that any code that uses Bronze will leak. It is for experimentation and exploration only.

## Usage

Sometimes, the requirements of ownership in Rust can be challenging to address. For example, implementing a doubly-linked list in Rust is notoriously hard because each node must be referenced twice. Also, especially for people who are learning Rust, it may be easier to avoid the _every variable has one owner_ restriction for the time being.

Bronze relaxes some of the restrictions that Rust has by introducing a new smart pointer type, GcRef<T>, which describes a pointer to a garbage-collected heap location. When using Bronze, data located on the _stack_ has all the usual Rust ownership requirements. However, Bronze allows moving data to the _heap_. If a value of type `T` is on the heap, Bronze allows an arbitrary number of references of type `GcRef<T>` to that value.

For example, without Bronze, we have to carefully manage references and lifetimes:

```
pub struct IntContainer {
    n: i32,
}

pub fn set(c: &mut IntContainer, n: i32) {
    c.n = n;
}

pub fn test() {
    let c1 = IntContainer{n: 42};
    let mut c2 = c1;

    // Can't use c1 anymore because it's been moved to c2
    set(&mut c2, 42);
}
```

With Bronze, types that are not `Copy` can be referenced through smart pointers, which _are_ `Copy`. Temporary immutable and immutable borrows can be obtained via `borrow()` and `borrow_mut`; these implement `Deref`, allowing convenient access.

```
//
#[derive(Trace, Finalize)]
pub struct IntContainer {
    n: i32,
}

pub fn set(mut c: GcRef<IntContainer>, n: i32) {
    (*c.borrow_mut()).n = n;
}

pub fn test() {
    let c1 = GcRef::new(IntContainer{n: 42});
    let c2 = c1;
    // Now c1 and c2 both reference the same object.

    set(c2, 42);
    set(c1, 43);
    // Now they both reference an object with value 43.
}
```

To create a new `GcRef<T>`, call `GcRef::new(e)`, where `e` is an expression whose value you want to be on the GC heap.

## Advanced Usage

If you need to remove data from the GC heap, you can use a `GcNullableRef` rather than a `GcRef`. You can create one with `Gc::new_nullable`. `GcNullableRef` is like `GcRef` but adds a method `remove`. The first call to `remove` returns an `Option` populated with the data that was previously in the GC heap. Future calls to `remove` return None.

## Safety

An earlier version of Bronze allowed users to obtain multiple mutable references to a GC object. This should no longer be possible; instead, Bronze dynamically tracks borrows (using a very similar mechanism to that used in RefCell). To obtain an immutable or mutable borrowed reference to a GC object, call `borrow()` or `borrow_mut()` on the `GcRef`.

## Experimental Implementation

This implementation is _experimental_. In particular, the collector will not run; be aware that you will eventually run out of memory. However, the present version is suitable for experimentation and prototyping.
