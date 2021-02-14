// Based on https://github.com/Manishearth/rust-gc/blob/master/gc_derive/src/lib.rs
//   which is licensed under MPL-2.0

use quote::quote;
use synstructure::{decl_derive, Structure};

decl_derive!([Trace, attributes(unsafe_ignore_trace)] => derive_trace);

fn derive_trace(mut s: Structure<'_>) -> proc_macro2::TokenStream {
    s.filter(|bi| {
        !bi.ast()
            .attrs
            .iter()
            .any(|attr| attr.path.is_ident("unsafe_ignore_trace"))
    });
    let trace_body = s.each(|bi| quote!(mark(#bi)));

    let trace_impl = s.unsafe_bound_impl(
        quote!(::bronze::GcTrace),
        quote! {
            #[inline] unsafe fn trace(&self) {
                #[allow(dead_code)]
                #[inline]
                unsafe fn mark<T: ::bronze::GcTrace + ?Sized>(it: &T) {
                    ::bronze::GcTrace::trace(it);
                }
                match *self { #trace_body }
            }
           
            // #[inline] fn finalize_glue(&self) {
            //     #[allow(dead_code)]
            //     #[inline]
            //     fn mark<T: ::bronze::GcTrace + ?Sized>(it: &T) {
            //         :::bronze::GcTrace::finalize_glue(it);
            //     }
            //     match *self { #trace_body }
            //     ::bronze::Finalize::finalize(self);
            // }
        },
    );

    // We also implement drop to prevent unsafe drop implementations on this
    // type and encourage people to use Finalize. This implementation will
    // call `Finalize::finalize` if it is safe to do so.
    // let drop_impl = s.unbound_impl(
    //     quote!(::std::ops::Drop),
    //     quote! {
    //         fn drop(&mut self) {
    //             if ::gc::finalizer_safe() {
    //                 ::gc::Finalize::finalize(self);
    //             }
    //         }
    //     },
    // );

    quote! {
        #trace_impl
        // #drop_impl
    }
}

// decl_derive!([Finalize] => derive_finalize);

// fn derive_finalize(s: Structure<'_>) -> proc_macro2::TokenStream {
//     s.unbound_impl(quote!(::bronze::Finalize), quote!())
// }