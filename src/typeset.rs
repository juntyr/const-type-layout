//! Helper module to compute the set of types that a type links to and expand it
//! into the complete type graph.

#[doc(hidden)]
pub trait ComputeSet: sealed::ComputeSet {
    const LEN: usize;

    type Output<H: ComputeTypeSet>: ExpandTypeSet;

    type TyHList: 'static + Copy + core::marker::Freeze;
    const TYS: &'static Self::TyHList;
}

mod sealed {
    pub trait ComputeSet {}

    impl ComputeSet for super::private::Empty {}
    impl<H2: super::ComputeTypeSet, T: ComputeSet> ComputeSet for super::private::Cons<H2, T> {}
}

type Set<H, T> = <T as ComputeSet>::Output<H>;

/// Computes the set of types that a type links to.
///
/// # Safety
///
/// It is only safe to implement this trait if it accurately includes
/// all inner component types that are referenced by this type's layout. Use
/// [`#[derive(TypeLayout)]`](const_type_layout_derive::TypeLayout) instead.
///
/// # Example
///
/// The struct `Foo` with `u8` and `u16` fields links to `u8` and `u16`:
///
/// ```rust
/// # #![feature(const_type_name)]
/// # #![feature(offset_of)]
/// # use const_type_layout::{
/// #    Field, MaybeUninhabited, TypeLayout, TypeLayoutInfo, TypeStructure,
/// # };
/// # use const_type_layout::inhabited;
/// # use const_type_layout::typeset::{ComputeTypeSet, ExpandTypeSet, tset};
/// struct Foo {
///     a: u8,
///     b: u16,
/// }
///
/// # unsafe impl TypeLayout for Foo {
/// #     type Inhabited = inhabited::all![u8, u16];
/// #
/// #     const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
/// #         name: ::core::any::type_name::<Self>(),
/// #         size: ::core::mem::size_of::<Self>(),
/// #         alignment: ::core::mem::align_of::<Self>(),
/// #         structure: TypeStructure::Struct {
/// #             repr: "",
/// #             fields: &[
/// #                 Field {
/// #                     name: "a",
/// #                     offset: MaybeUninhabited::new::<u8>(::core::mem::offset_of!(Self, a)),
/// #                     ty: ::core::any::type_name::<u8>(),
/// #                 },
/// #                 Field {
/// #                     name: "b",
/// #                     offset: MaybeUninhabited::new::<u16>(::core::mem::offset_of!(Self, b)),
/// #                     ty: ::core::any::type_name::<u16>(),
/// #                 },
/// #             ],
/// #         },
/// #     };
/// # }
///
/// unsafe impl ComputeTypeSet for Foo {
///     type Output<T: ExpandTypeSet> = tset![u8, u16];
/// }
/// ```
///
/// Note that to you implement [`ComputeTypeSet`] you must also implement
/// [`crate::TypeLayout`] for it.
pub unsafe trait ComputeTypeSet: crate::TypeLayout {
    /// Extend the set `T` into a (larger) set containing also the types this
    /// type links to.
    ///
    /// Enums implementing [`crate::TypeLayout`] and [`ComputeTypeSet`]
    /// manually should include [`crate::ExtractDiscriminant::Discriminant`] in
    /// their [`ComputeTypeSet::Output`] using the [`tset`] helper macro.
    type Output<T: ExpandTypeSet>: ExpandTypeSet;
}

/// Helper macro to expand a list of types, e.g. `H, R1, R2`, and an optional
/// tail, `.. @ T`, into a set of types.
///
/// This macro is used when implementing the [`ComputeTypeSet::Output`]
/// associated type to specify the list of types a type links to.
pub macro tset {
    () => { private::Empty },
    (.. @ $T:tt) => { $T },
    ($H:ty $(, $R:ty)*) => {
        Set<$H, tset![$($R),*]>
    },
    ($H:ty, $($R:ty,)* .. @ $T:ty ) => {
        Set<$H, tset![$($R,)* .. @ $T]>
    },
}

#[doc(hidden)]
pub trait ExpandTypeSet: ComputeSet {
    type Output<T: ExpandTypeSet>: ExpandTypeSet;
}

impl ExpandTypeSet for private::Empty {
    type Output<T: ExpandTypeSet> = T;
}

impl<H: ComputeTypeSet, T: ExpandTypeSet> ExpandTypeSet for private::Cons<H, T> {
    type Output<R: ExpandTypeSet> =
        <T as ExpandTypeSet>::Output<Set<H, <H as ComputeTypeSet>::Output<R>>>;
}

#[doc(hidden)]
pub trait TypeSetFixedPoint: ExpandTypeSet {
    type Output: ExpandTypeSet;
}

impl<T: ExpandTypeSet> TypeSetFixedPoint for T {
    type Output = <T as private::ComputeTypeSetFixedPoint<
        <T as ExpandTypeSet>::Output<private::Empty>,
    >>::Output;
}

mod private {
    use super::{sealed, ComputeSet, ComputeTypeSet, ExpandTypeSet, Set};

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Empty;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Cons<H, T> {
        head: H,
        tail: T,
    }

    impl ComputeSet for Empty {
        type Output<H: ComputeTypeSet> = Cons<H, Self>;
        type TyHList = Self;

        const LEN: usize = 0;
        const TYS: &'static Self::TyHList = &Self;
    }

    impl<H2: ComputeTypeSet, T: ExpandTypeSet> ComputeSet for Cons<H2, T> {
        type Output<H1: ComputeTypeSet> = <Self as ComputeCons<H1>>::Output;
        type TyHList = Cons<&'static crate::TypeLayoutInfo<'static>, T::TyHList>;

        const LEN: usize = T::LEN + 1;
        const TYS: &'static Self::TyHList = &Cons {
            head: &H2::TYPE_LAYOUT,
            tail: *T::TYS,
        };
    }

    pub trait ComputeCons<H: ComputeTypeSet>: sealed::ComputeSet {
        type Output: ExpandTypeSet;
    }

    impl<H: ComputeTypeSet> ComputeCons<H> for Empty {
        type Output = Cons<H, Self>;
    }

    impl<H: ComputeTypeSet, T: ExpandTypeSet> ComputeCons<H> for Cons<H, T> {
        type Output = Self;
    }

    impl<H1: ComputeTypeSet, H2: ComputeTypeSet, T: ExpandTypeSet> ComputeCons<H1> for Cons<H2, T> {
        default type Output = Cons<H2, Set<H1, T>>;
    }

    pub trait ComputeTypeSetFixedPoint<E: ExpandTypeSet>: ExpandTypeSet {
        type Output: ExpandTypeSet;
    }

    impl<T: ExpandTypeSet, E: ExpandTypeSet> ComputeTypeSetFixedPoint<E> for T {
        default type Output = <E as ComputeTypeSetFixedPoint<<E as ExpandTypeSet>::Output<Empty>>>::Output;
    }

    trait True {}
    struct Assert<const ASSERT: bool>;
    impl True for Assert<true> {}

    impl<T: ExpandTypeSet, E: ExpandTypeSet> ComputeTypeSetFixedPoint<E> for T
    where
        Assert<{ T::LEN == E::LEN }>: True,
    {
        type Output = T;
    }
}

pub(super) type TypeSet<T> =
    <Set<T, <T as ComputeTypeSet>::Output<private::Empty>> as TypeSetFixedPoint>::Output;
