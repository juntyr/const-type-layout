//! Helper module to compute the set of types that a type links to and expand it
//! into the complete type graph.

use core::marker::Freeze;

use crate::TypeLayout;

const fn foo<T: Copy + core::marker::Freeze>() -> &'static T {
    const fn bar<T: Copy + core::marker::Freeze>() -> T {
        panic!()
    }

    &const { bar::<T>() }
}

pub mod foo {
    use core::{marker::Freeze, mem::MaybeUninit, ops::Deref};

    use crate::{Field, TypeLayout, TypeLayoutInfo, Variant};

    use super::{ComputeSet, ComputeTypeSet, ExpandTypeSet, private::{Cons, Empty}};

    struct Node<
        'a,
        'b,
        F: Deref<Target = [Field<'a>]> = &'a [Field<'a>],
        D: Deref<Target = [u8]> = &'a [u8],
        V: Deref<Target = [Variant<'a, F, D>]> = &'a [Variant<'a, F, D>],
    > {
        ty: TypeLayoutInfo<'a, F, D, V>,
        next: Option<&'b Self>,
    }

    trait HList: 'static + Copy + Freeze {
        const LEN: usize;
    }

    impl HList for Empty {
        const LEN: usize = 0;
    }

    impl<H: 'static + Copy + Freeze, T: HList> HList for Cons<H, T> {
        const LEN: usize = 1 + T::LEN;
    }

    const fn str_eq(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let (a, b) = (a.as_bytes(), b.as_bytes());

        let mut i = 0;

        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }

            i += 1;
        }

        true
    }

    pub const fn type_layout_graph<T: TypeLayout + ComputeTypeSet>() -> &'static [TypeLayoutInfo<'static>] {
        expand_type_layout_graph::<T, Empty, T, Empty>(None)
    }

    const fn expand_type_layout_graph<A: TypeLayout + ComputeTypeSet, G: HList, T: ComputeTypeSet, S: ExpandTypeSet>(tys: Option<&Node>) -> &'static [TypeLayoutInfo<'static>] {
        const fn fill_type_layout_graph<A: TypeLayout + ComputeTypeSet, G: HList>() -> &'static [TypeLayoutInfo<'static>] {
            const fn fill_type_layout_graph_erased<A: TypeLayout + ComputeTypeSet, G: HList>() -> G {
                const fn expand_type_layout_graph_erased<T: ComputeTypeSet, S: ExpandTypeSet>(tys: &mut [MaybeUninit<TypeLayoutInfo<'static>>], tys_len: usize) -> usize {
                    let info = T::TYPE_LAYOUT;
                    let mut i = 0;
                    while i < tys_len {
                        if str_eq(unsafe { tys[i].assume_init_ref() }.name, info.name) {
                            if S::IS_EMPTY {
                                return tys_len;
                            }
                            return expand_type_layout_graph_erased::<S::Head, S::Tail>(tys, tys_len);
                        }
                        i += 1;
                    }
                    if tys_len >= tys.len() {
                        panic!("bug: type layout graph too small");
                    }
                    tys[tys_len] = MaybeUninit::new(info);
                    if <T::Output<S> as ExpandTypeSet>::IS_EMPTY {
                        return tys_len + 1;
                    }
                    expand_type_layout_graph_erased::<<T::Output<S> as ComputeSet>::Head, <T::Output<S> as ComputeSet>::Tail>(tys, tys_len + 1)
                }

                let mut graph = MaybeUninit::<G>::uninit();

                let graph_slice = unsafe { core::slice::from_raw_parts_mut(core::ptr::from_mut(&mut graph).cast(), G::LEN) };
                let graph_len = expand_type_layout_graph_erased::<A, Empty>(graph_slice, 0);

                if graph_len != G::LEN {
                    panic!("bug: initialized graph has the wrong size");
                }

                unsafe { graph.assume_init() }
            }
            
            const fn fill_type_layout_graph_erased_reference<A: TypeLayout + ComputeTypeSet, G: HList>() -> &'static G {
                &const { fill_type_layout_graph_erased::<A, G>() }
            }

            unsafe { core::slice::from_raw_parts(core::ptr::from_ref(const {
                fill_type_layout_graph_erased_reference::<A, G>()
            }).cast(), G::LEN) }
        }

        let info = T::TYPE_LAYOUT;
        let mut it = &tys;
        while let Some(i) = it {
            if str_eq(i.ty.name, info.name) {
                if S::IS_EMPTY {
                    return fill_type_layout_graph::<A, G>();
                }
                return expand_type_layout_graph::<A, G, S::Head, S::Tail>(tys);
            }

            it = &i.next;
        }
        let mut cons = Node {
            ty: info,
            next: tys,
        };
        if <T::Output<S> as ExpandTypeSet>::IS_EMPTY {
            return fill_type_layout_graph::<A, Cons<TypeLayoutInfo<'static>, G>>();
        }
        expand_type_layout_graph::<A, Cons<TypeLayoutInfo<'static>, G>, <T::Output<S> as ComputeSet>::Head, <T::Output<S> as ComputeSet>::Tail>(Some(&mut cons))
    }
}

#[doc(hidden)]
pub trait ComputeSet: sealed::ComputeSet {
    const LEN: usize;
    type Len;

    type Output<H: ComputeTypeSet>: ExpandTypeSet;

    type Head: ComputeTypeSet;
    type Tail: ExpandTypeSet;

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
/// #     const INHABITED: MaybeUninhabited = inhabited::all![u8, u16];
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
    /// manually should include [`core::mem::Discriminant<Self>`] in
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
    const IS_EMPTY: bool;

    type Output<T: ExpandTypeSet>: ExpandTypeSet;
}

impl ExpandTypeSet for private::Empty {
    const IS_EMPTY: bool = true;

    type Output<T: ExpandTypeSet> = T;
}

impl<H: ComputeTypeSet, T: ExpandTypeSet> ExpandTypeSet for private::Cons<H, T> {
    const IS_EMPTY: bool = false;

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
        type Len = Self;
        type Output<H: ComputeTypeSet> = Cons<H, Self>;
        type TyHList = Self;

        type Head = (); // FIXME
        type Tail = Self;

        const LEN: usize = 0;
        const TYS: &'static Self::TyHList = &Self;
    }

    impl<H2: ComputeTypeSet, T: ExpandTypeSet> ComputeSet for Cons<H2, T> {
        type Len = Cons<(), T::Len>;
        type Output<H1: ComputeTypeSet> = <Self as ComputeCons<H1>>::Output;
        type TyHList = Cons<&'static crate::TypeLayoutInfo<'static>, T::TyHList>;

        type Head = H2;
        type Tail = T;

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

    trait Same<T> {}
    impl<T> Same<T> for T {}

    impl<T: ExpandTypeSet, E: ExpandTypeSet> ComputeTypeSetFixedPoint<E> for T
    where
        T::Len: Same<E::Len>,
    {
        type Output = Self;
    }

    impl<T: ExpandTypeSet, E: ExpandTypeSet> ComputeTypeSetFixedPoint<E> for T {
        default type Output = <E as ComputeTypeSetFixedPoint<<E as ExpandTypeSet>::Output<Empty>>>::Output;
    }
}

pub(super) type TypeSet<T> =
    <Set<T, <T as ComputeTypeSet>::Output<private::Empty>> as TypeSetFixedPoint>::Output;
