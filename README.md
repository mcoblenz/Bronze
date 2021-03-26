# Bronze

## Installation
To use Bronze, you must use the nightly Rust tools. To install them, run `rustup default nightly` on the command line.

## Usage
Bronze provides a new smart pointer type, GcRef<T>, which describes a pointer to a garbage-collected heap location.

To create a new `GcRef<T>`, call `GcRef::new(e)`, where `e` is an expression whose value you want to be on the GC heap. Then, you can use `as_ref` and `as_mut` to obtain references to the data in the GC heap. 

If you need to remove data from the GC heap, you can use a `GcNullableRef` rather than a `GcRef`. You can create one with `Gc::new_nullable`. `GcNullableRef` is like `GcRef` but adds a method `remove`. The first call to `remove` returns an `Option` populated with the data that was previously in the GC heap. Future calls to `remove` return None.

This implementation is *experimental*. In particular, the collector will not run; be aware that you will eventually run out of memory. However, the present version is suitable for experimentation and prototyping.