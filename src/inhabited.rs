//! Helper module to compute whether a combination of types implementing
//! [`crate::TypeLayout`] are [inhabited] or [uninhabited].
//!
//! [inhabited]: https://doc.rust-lang.org/reference/glossary.html#inhabited
//! [uninhabited]: https://doc.rust-lang.org/reference/glossary.html#uninhabited

#[allow(non_upper_case_globals)]
/// Marker used to specify that a type implementing [`crate::TypeLayout`] is
/// [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited).
pub const Inhabited: crate::MaybeUninhabited = crate::MaybeUninhabited::Inhabited(());

/// Marker used to specify that a type implementing [`crate::TypeLayout`] is
/// [uninhabited](https://doc.rust-lang.org/reference/glossary.html#uninhabited).
pub use crate::MaybeUninhabited::Uninhabited;

/// Helper macro to compute whether all of a list of types, all implementing
/// [`crate::TypeLayout`], e.g. `[T, U, V]`, are [inhabited].
///
/// For instance, a struct is inhabited iff all of its fields are inhabited.
/// The empty list of types is inhabited. This macro resolves into either
/// [`Inhabited`] or [`Uninhabited`].
///
/// [inhabited]: https://doc.rust-lang.org/reference/glossary.html#inhabited
#[macro_export]
macro_rules! all {
    () => { $crate::inhabited::Inhabited };
    ($L:ty $(, $R:ty)*) => {
        <$L as $crate::TypeLayout>::INHABITED.and(
            $crate::inhabited::all![$($R),*]
        )
    };
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
#[macro_export]
macro_rules! any {
    () => { $crate::inhabited::Uninhabited };
    ($L:ty $(, $R:ty)*) => {
        <$L as $crate::TypeLayout>::INHABITED.or(
            $crate::inhabited::any![$($R),*]
        )
    };
}

pub use all;
pub use any;
