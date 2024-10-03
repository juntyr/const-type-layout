//! Helper module to compute whether a combination of types implementing
//! [`crate::TypeLayout`] are [inhabited] or [uninhabited].
//!
//! [inhabited]: https://doc.rust-lang.org/reference/glossary.html#inhabited
//! [uninhabited]: https://doc.rust-lang.org/reference/glossary.html#uninhabited

/// Helper macro to compute whether
///
/// - all of a list of types, all implementing [`crate::TypeLayout`], e.g. `[T,
///   U, V]`,
/// - all of a list of braced expressions of type [`crate::MaybeUninhabited`],
///   e.g. `[{ all![] }, { any![] }, { all![] }]`,
///
/// are [inhabited].
///
/// For instance, a struct is inhabited iff all of its fields are inhabited.
/// The empty list of types is inhabited. This macro resolves into either
/// [`crate::MaybeUninhabited::Inhabited`] or
/// [`crate::MaybeUninhabited::Uninhabited`].
///
/// [inhabited]: https://doc.rust-lang.org/reference/glossary.html#inhabited
#[macro_export]
#[doc(hidden)]
macro_rules! all {
    () => { $crate::MaybeUninhabited::Inhabited(()) };
    ($L:ty $(, $R:ty)*) => {
        <$L as $crate::TypeLayout>::INHABITED.and(
            $crate::inhabited::all![$($R),*]
        )
    };
    ({ $L:expr } $(, { $R:expr })*) => {
        $crate::MaybeUninhabited::and(
            $L, $crate::inhabited::all![$({ $R }),*]
        )
    };
}

/// Helper macro to compute whether
///
/// - any of a list of types, all implementing [`crate::TypeLayout`], e.g. `[T,
///   U, V]`,
/// - any of a list of braced expressions of type [`crate::MaybeUninhabited`],
///   e.g. `[{ all![] }, { any![] }, { all![] }]`,
///
/// is [inhabited].
///
/// For instance, an enum is inhabited iff any of its variants is inhabited.
/// The empty list of types is [uninhabited]. This macro resolves into either
/// [`crate::MaybeUninhabited::Inhabited`] or
/// [`crate::MaybeUninhabited::Uninhabited`].
///
/// [inhabited]: https://doc.rust-lang.org/reference/glossary.html#inhabited
/// [uninhabited]: https://doc.rust-lang.org/reference/glossary.html#uninhabited
#[macro_export]
#[doc(hidden)]
macro_rules! any {
    () => { $crate::MaybeUninhabited::Uninhabited };
    ($L:ty $(, $R:ty)*) => {
        <$L as $crate::TypeLayout>::INHABITED.or(
            $crate::inhabited::any![$($R),*]
        )
    };
    ({ $L:expr } $(, { $R:expr })*) => {
        $crate::MaybeUninhabited::or(
            $L, $crate::inhabited::any![$({ $R }),*]
        )
    };
}

#[doc(inline)]
pub use {all, any};
