//! Helper module to compute the set of types that a type links to and expand it
//! into the complete type graph.

use core::{marker::Freeze, mem::MaybeUninit};

use crate::{TypeLayout, TypeLayoutInfo};

struct Node<'a> {
    ty: TypeLayoutInfo<'static>,
    next: Option<&'a Self>,
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

#[must_use]
pub const fn type_layout_graph<T: TypeLayout + ComputeTypeSet>(
) -> &'static [TypeLayoutInfo<'static>] {
    expand_type_layout_graph::<T, private::Empty, T, private::Empty>(None)
}

const fn expand_type_layout_graph<
    A: TypeLayout + ComputeTypeSet,
    G: 'static + Copy + Freeze + private::HList,
    T: ComputeTypeSet,
    S: ExpandTypeHList,
>(
    tys: Option<&Node>,
) -> &'static [TypeLayoutInfo<'static>] {
    const fn fill_type_layout_graph<
        A: TypeLayout + ComputeTypeSet,
        G: 'static + Copy + Freeze + private::HList,
    >() -> &'static [TypeLayoutInfo<'static>] {
        const fn fill_type_layout_graph_erased<
            A: TypeLayout + ComputeTypeSet,
            G: private::HList,
        >() -> G {
            const fn expand_type_layout_graph_erased<T: ComputeTypeSet, S: ExpandTypeHList>(
                tys: &mut [MaybeUninit<TypeLayoutInfo<'static>>],
                tys_len: usize,
            ) -> usize {
                let info = T::TYPE_LAYOUT;
                let mut i = 0;
                while i < tys_len {
                    if str_eq(unsafe { tys[i].assume_init_ref() }.name, info.name) {
                        if S::LEN == 0 {
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
                if <T::Output<S> as private::HList>::LEN == 0 {
                    return tys_len + 1;
                }
                expand_type_layout_graph_erased::<
                    <T::Output<S> as private::TypeHList>::Head,
                    <T::Output<S> as private::TypeHList>::Tail,
                >(tys, tys_len + 1)
            }

            let mut graph = MaybeUninit::<G>::uninit();

            let graph_slice = unsafe {
                core::slice::from_raw_parts_mut(core::ptr::from_mut(&mut graph).cast(), G::LEN)
            };
            let graph_len = expand_type_layout_graph_erased::<A, private::Empty>(graph_slice, 0);

            if graph_len != G::LEN {
                panic!("bug: initialized graph has the wrong size");
            }

            unsafe { graph.assume_init() }
        }

        const fn fill_type_layout_graph_erased_reference<
            A: TypeLayout + ComputeTypeSet,
            G: 'static + Copy + Freeze + private::HList,
        >() -> &'static G {
            &const { fill_type_layout_graph_erased::<A, G>() }
        }

        unsafe {
            core::slice::from_raw_parts(
                core::ptr::from_ref(const { fill_type_layout_graph_erased_reference::<A, G>() })
                    .cast(),
                G::LEN,
            )
        }
    }

    let info = T::TYPE_LAYOUT;
    let mut it = &tys;
    while let Some(i) = it {
        if str_eq(i.ty.name, info.name) {
            if S::LEN == 0 {
                return fill_type_layout_graph::<A, G>();
            }
            return expand_type_layout_graph::<A, G, S::Head, S::Tail>(tys);
        }

        it = &i.next;
    }
    if <T::Output<S> as private::HList>::LEN == 0 {
        return fill_type_layout_graph::<A, private::Cons<TypeLayoutInfo<'static>, G>>();
    }
    expand_type_layout_graph::<
        A,
        private::Cons<TypeLayoutInfo<'static>, G>,
        <T::Output<S> as private::TypeHList>::Head,
        <T::Output<S> as private::TypeHList>::Tail,
    >(Some(&Node {
        ty: info,
        next: tys,
    }))
}

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
/// # use const_type_layout::typeset::{ComputeTypeSet, ExpandTypeHList, tset};
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
///     type Output<T: ExpandTypeHList> = tset![u8, u16];
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
    type Output<T: ExpandTypeHList>: ExpandTypeHList;
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
        private::Cons<$H, tset![$($R),*]>
    },
    ($H:ty, $($R:ty,)* .. @ $T:ty ) => {
        private::Cons<$H, tset![$($R,)* .. @ $T]>
    },
}

#[doc(hidden)]
pub trait ExpandTypeHList: private::TypeHList {
    type Output<T: ExpandTypeHList>: ExpandTypeHList;
}

impl ExpandTypeHList for private::Empty {
    type Output<T: ExpandTypeHList> = T;
}

impl<H: ComputeTypeSet, T: ExpandTypeHList> ExpandTypeHList for private::Cons<H, T> {
    type Output<R: ExpandTypeHList> =
        <T as ExpandTypeHList>::Output<private::Cons<H, <H as ComputeTypeSet>::Output<R>>>;
}

mod private {
    use super::{ComputeTypeSet, ExpandTypeHList};

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Empty;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Cons<H, T> {
        head: H,
        tail: T,
    }

    pub trait HList {
        const LEN: usize;
    }

    impl HList for Empty {
        const LEN: usize = 0;
    }

    impl<H, T: HList> HList for Cons<H, T> {
        const LEN: usize = 1 + T::LEN;
    }

    pub trait TypeHList: HList {
        type Head: ComputeTypeSet;
        type Tail: ExpandTypeHList;
    }

    impl TypeHList for Empty {
        // Empty's head can be anything since we never use it
        type Head = ();
        type Tail = Self;
    }

    impl<H2: ComputeTypeSet, T: ExpandTypeHList> TypeHList for Cons<H2, T> {
        type Head = H2;
        type Tail = T;
    }
}
