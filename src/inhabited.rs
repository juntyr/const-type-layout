//! Helper module to compute whether a combination of types implementing
//! [`crate::TypeLayout`] are [inhabited] or [uninhabited].
//!
//! [inhabited]: https://doc.rust-lang.org/reference/glossary.html#inhabited
//! [uninhabited]: https://doc.rust-lang.org/reference/glossary.html#uninhabited

#![allow(clippy::undocumented_unsafe_blocks)]
#![allow(missing_docs)] // FIXME

/// Helper macro to compute whether all of a list of types, all implementing
/// [`crate::TypeLayout`], e.g. `[T, U, V]`, are [inhabited].
///
/// For instance, a struct is inhabited iff all of its fields are inhabited.
/// The empty list of types is inhabited. This macro resolves into either
/// [`Inhabited`] or [`Uninhabited`].
///
/// [inhabited]: https://doc.rust-lang.org/reference/glossary.html#inhabited
pub macro all {
    () => { $crate::MaybeUninhabited::Inhabited(()) },
    ($L:ty $(, $R:ty)*) => {
        <$L as $crate::TypeLayout>::INHABITED.and(all![$($R),*])
    },
}

/// Helper macro to compute whether any of a list of types, all implementing
/// [`crate::TypeLayout`], e.g. `[T, U, V]`, is [inhabited].
///
/// For instance, an enum is inhabited iff any of its variants is inhabited.
/// The empty list of types is [uninhabited]. This macro resolves into either
/// [`Inhabited`] or [`Uninhabited`].
///
/// [inhabited]: https://doc.rust-lang.org/reference/glossary.html#inhabited
/// [uninhabited]: https://doc.rust-lang.org/reference/glossary.html#uninhabited
pub macro any {
    () => { $crate::MaybeUninhabited::Uninhabited },
    ($L:ty $(, $R:ty)*) => {
        <$L as $crate::TypeLayout>::INHABITED.or(any![$($R),*])
    },
}
