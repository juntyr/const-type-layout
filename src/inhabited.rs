//! Helper module to compute whether a combination of types implementing
//! [`crate::TypeLayout`] are
//! [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited) or
//! [uninhabited](https://doc.rust-lang.org/reference/glossary.html#uninhabited).

#![allow(clippy::undocumented_unsafe_blocks)]

/// Marker type used to specify that a type implementing
/// [`crate::TypeLayout::Inhabited`] is
/// [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited).
pub struct Inhabited;

unsafe impl crate::TypeLayout for Inhabited {
    type Inhabited = Self;

    const TYPE_LAYOUT: crate::TypeLayoutInfo<'static> = crate::TypeLayoutInfo {
        ty: crate::TypeRef::of::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: crate::TypeStructure::Struct {
            repr: "",
            fields: &[],
        },
    };
}

unsafe impl crate::typeset::ComputeTypeSet for Inhabited {
    type Output<T: crate::typeset::ExpandTypeSet> = crate::typeset::tset![.. @ T];
}

#[allow(clippy::empty_enum)]
/// Marker type used to specify that a type implementing
/// [`crate::TypeLayout::Inhabited`] is
/// [uninhabited](https://doc.rust-lang.org/reference/glossary.html#uninhabited).
pub enum Uninhabited {}

unsafe impl crate::TypeLayout for Uninhabited {
    type Inhabited = Self;

    const TYPE_LAYOUT: crate::TypeLayoutInfo<'static> = crate::TypeLayoutInfo {
        ty: crate::TypeRef::of::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: crate::TypeStructure::Enum {
            repr: "",
            variants: &[],
        },
    };
}

unsafe impl crate::typeset::ComputeTypeSet for Uninhabited {
    type Output<T: crate::typeset::ExpandTypeSet> = crate::typeset::tset![.. @ T];
}

#[allow(clippy::module_name_repetitions)]
#[doc(hidden)]
pub trait ComputeInhabited: sealed::ComputeInhabited {
    type Output: OutputMaybeInhabited;
}

#[allow(clippy::module_name_repetitions)]
#[doc(hidden)]
pub trait OutputMaybeInhabited:
    ComputeInhabited + crate::TypeGraphLayout + sealed::OutputMaybeInhabited
{
}

mod sealed {
    pub trait ComputeInhabited {}
    pub trait OutputMaybeInhabited {}
}

impl sealed::ComputeInhabited for Inhabited {}
impl ComputeInhabited for Inhabited {
    type Output = Self;
}
impl sealed::OutputMaybeInhabited for Inhabited {}
impl OutputMaybeInhabited for Inhabited {}

impl sealed::ComputeInhabited for Uninhabited {}
impl ComputeInhabited for Uninhabited {
    type Output = Self;
}
impl sealed::OutputMaybeInhabited for Uninhabited {}
impl OutputMaybeInhabited for Uninhabited {}

mod logical {
    use super::{sealed, ComputeInhabited, Inhabited, Uninhabited};

    pub struct And<L: ComputeInhabited, R: ComputeInhabited> {
        _left: L,
        _right: R,
    }

    impl<L: ComputeInhabited, R: ComputeInhabited> sealed::ComputeInhabited for And<L, R> {}
    impl<L: ComputeInhabited, R: ComputeInhabited> ComputeInhabited for And<L, R> {
        default type Output = Uninhabited;
    }

    impl<L: ComputeInhabited<Output = Inhabited>, R: ComputeInhabited<Output = Inhabited>>
        ComputeInhabited for And<L, R>
    {
        type Output = Inhabited;
    }

    pub struct Or<L: ComputeInhabited, R: ComputeInhabited> {
        _left: L,
        _right: R,
    }

    impl<L: ComputeInhabited, R: ComputeInhabited> sealed::ComputeInhabited for Or<L, R> {}
    impl<L: ComputeInhabited, R: ComputeInhabited> ComputeInhabited for Or<L, R> {
        default type Output = Inhabited;
    }

    impl<L: ComputeInhabited<Output = Uninhabited>, R: ComputeInhabited<Output = Uninhabited>>
        ComputeInhabited for Or<L, R>
    {
        type Output = Uninhabited;
    }
}

/// Helper macro to compute whether all of a list of types, all implementing
/// [`crate::TypeLayout`], e.g. `[T, U, V]`, are
/// [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited).
/// For instance, a struct is inhabited iff all of its fields are inhabited.
/// The empty list of types is inhabited. This macro resolves into either
/// [`Inhabited`] or [`Uninhabited`].
pub macro all {
    () => { Inhabited },
    ($L:ty $(, $R:ty)*) => {
        <logical::And<<$L as $crate::TypeLayout>::Inhabited, all![$($R),*]> as ComputeInhabited>::Output
    },
}

/// Helper macro to compute whether any of a list of types, all implementing
/// [`crate::TypeLayout`], e.g. `[T, U, V]`, is
/// [inhabited](https://doc.rust-lang.org/reference/glossary.html#inhabited).
/// For instance, an enum is inhabited iff any of its variants is inhabited.
/// The empty list of types is
/// [uninhabited](https://doc.rust-lang.org/reference/glossary.html#uninhabited).
/// This macro resolves into either [`Inhabited`] or [`Uninhabited`].
pub macro any {
    () => { Uninhabited },
    ($L:ty $(, $R:ty)*) => {
        <logical::Or<<$L as $crate::TypeLayout>::Inhabited, any![$($R),*]> as ComputeInhabited>::Output
    },
}
