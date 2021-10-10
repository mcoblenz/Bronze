# Bronze

## Installation
To use Bronze, you must use the nightly Rust tools. To install them, run `rustup default nightly` on the command line.

## Usage
Sometimes, the requirements of ownership in Rust can be challenging to address. For example, implementing a doubly-linked list in Rust is notoriously hard because each node must be referenced twice. Also, especially for people who are learning Rust, it may be easier to avoid the *every variable has one owner* restriction for the time being.

Bronze relaxes some of the restrictions that Rust has by introducing a new smart pointer type, GcRef<T>, which describes a pointer to a garbage-collected heap location. When using Bronze, data located on the *stack* has all the usual Rust ownership requirements. However, Bronze allows moving data to the *heap*. If a value of type `T` is on the heap, Bronze allows an arbitrary number of references of type `GcRef<T>` to that value.

For example, without Bronze, we have to carefully manage references and lifetimes:

````
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
````

With Bronze, types that are not `Copy` can be freely referenced through smart pointers, which *are* `Copy`:

````
// 
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
````

Because `GcRef` implements `Deref`, and because Rust automatically inserts `*` when needed, you can generally treat `GcRef<T>` as if it were a `T` directly. For example, in `set()` above, the body assigns to `c.n`. This implicitly means to dereference the pointer and assign to `n` inside the referenced value. If you like, you can instead call `as_ref()` and `as_mut()` to obtain a reference or mutable reference to the data in the GC heap. 


To create a new `GcRef<T>`, call `GcRef::new(e)`, where `e` is an expression whose value you want to be on the GC heap.

## Advanced Usage
If you need to remove data from the GC heap, you can use a `GcNullableRef` rather than a `GcRef`. You can create one with `Gc::new_nullable`. `GcNullableRef` is like `GcRef` but adds a method `remove`. The first call to `remove` returns an `Option` populated with the data that was previously in the GC heap. Future calls to `remove` return None.

## Experimental Implementation
This implementation is *experimental*. The 'main' branch has a collector, but it only works in limited cases is not general enough to work with YOUR code. The 'API-only' branch has the collector disabled; be aware that you will eventually run out of memory. However, the present version is suitable for experimentation and prototyping.

An important aspect of the experimental nature is that, for now, it is possible with Bronze to create multiple mutable references to the same data. Obviously, this is unsafe in Rust. For now, we assume that users will encapsulate their data structures appropriately and avoid this; in the future, we should explore enforcement mechanisms to make this safe in general. One approach might be akin to how RefCell keeps track of outstanding references; another might be akin to Stacked Borrows.