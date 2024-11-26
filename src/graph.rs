#![allow(missing_docs)] // FIXME

use core::{marker::Freeze, mem::MaybeUninit};

use crate::{TypeLayout, TypeLayoutInfo};

pub(crate) use private::TypeHList;

pub macro hlist {
    () => { private::Empty },
    ($H:ty $(, $R:ty)*) => {
        private::Cons<$H, hlist![$($R),*]>
    },
}

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub const fn type_layout_graph<T: TypeLayout>() -> &'static [&'static TypeLayoutInfo<'static>] {
    expand_type_layout_graph::<T, private::Empty, T, private::Empty>(None)
}

const fn expand_type_layout_graph<
    A: TypeLayout,
    G: 'static + Copy + Freeze + private::HList,
    T: TypeLayout,
    S: private::TypeHList,
>(
    tys: Option<&Node>,
) -> &'static [&'static TypeLayoutInfo<'static>] {
    const fn fill_type_layout_graph<A: TypeLayout, G: 'static + Copy + Freeze + private::HList>(
    ) -> &'static [&'static TypeLayoutInfo<'static>] {
        const fn fill_type_layout_graph_erased<A: TypeLayout, G: private::HList>() -> G {
            const fn expand_type_layout_graph_erased<T: TypeLayout, S: private::TypeHList>(
                tys: &mut [MaybeUninit<&'static TypeLayoutInfo<'static>>],
                tys_len: usize,
            ) -> usize {
                let info = &T::TYPE_LAYOUT;
                let mut i = 0;
                while i < tys_len {
                    // Safety: tys has been initialized for 0..tys_len
                    if str_eq(unsafe { tys[i].assume_init_ref() }.name, info.name) {
                        if S::LEN == 0 {
                            return tys_len;
                        }
                        return expand_type_layout_graph_erased::<S::Head, S::Tail>(tys, tys_len);
                    }
                    i += 1;
                }
                assert!(tys_len < tys.len(), "bug: type layout graph too small");
                tys[tys_len] = MaybeUninit::new(info);
                if <<T::TypeGraphEdges as private::TypeHList>::Concat<S> as private::HList>::LEN
                    == 0
                {
                    return tys_len + 1;
                }
                expand_type_layout_graph_erased::<
                    <<T::TypeGraphEdges as private::TypeHList>::Concat<S> as private::TypeHList>::Head,
                    <<T::TypeGraphEdges as private::TypeHList>::Concat<S> as private::TypeHList>::Tail,
                >(tys, tys_len + 1)
            }

            let mut graph = MaybeUninit::<G>::uninit();

            // SAFETY:
            // - HList is a sealed trait and is constructed here to be made of only Cons of
            //   TypeLayoutInfo and Empty
            // - Cons is a repr(C) struct with a head followed by a tail, Empty is a
            //   zero-sized repr(C) struct
            // - the HList is layout-equivalent to an array of the same length as HList::LEN
            let graph_slice = unsafe {
                core::slice::from_raw_parts_mut(core::ptr::from_mut(&mut graph).cast(), G::LEN)
            };
            let graph_len = expand_type_layout_graph_erased::<A, private::Empty>(graph_slice, 0);

            assert!(
                graph_len == G::LEN,
                "bug: initialized graph has the wrong size"
            );

            // Safety: we have just checked that all array elements have been initialized
            unsafe { graph.assume_init() }
        }

        const fn fill_type_layout_graph_erased_reference<
            A: TypeLayout,
            G: 'static + Copy + Freeze + private::HList,
        >() -> &'static G {
            &const { fill_type_layout_graph_erased::<A, G>() }
        }

        // SAFETY:
        // - HList is a sealed trait and is constructed here to be made of only Cons of
        //   TypeLayoutInfo and Empty
        // - Cons is a repr(C) struct with a head followed by a tail, Empty is a
        //   zero-sized repr(C) struct
        // - the HList is layout-equivalent to an array of the same length as HList::LEN
        // - fill_type_layout_graph_erased_reference provides a static non-dangling
        //   reference that we can use to produce the data pointer for a slice
        unsafe {
            core::slice::from_raw_parts(
                core::ptr::from_ref(const { fill_type_layout_graph_erased_reference::<A, G>() })
                    .cast(),
                G::LEN,
            )
        }
    }

    let info = &T::TYPE_LAYOUT;
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
    if <<T::TypeGraphEdges as private::TypeHList>::Concat<S> as private::HList>::LEN == 0 {
        return fill_type_layout_graph::<A, private::Cons<&'static TypeLayoutInfo<'static>, G>>();
    }
    expand_type_layout_graph::<
        A,
        private::Cons<&'static TypeLayoutInfo<'static>, G>,
        <<T::TypeGraphEdges as private::TypeHList>::Concat<S> as private::TypeHList>::Head,
        <<T::TypeGraphEdges as private::TypeHList>::Concat<S> as private::TypeHList>::Tail,
    >(Some(&Node {
        ty: info,
        next: tys,
    }))
}

struct Node<'a> {
    ty: &'static TypeLayoutInfo<'static>,
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

mod private {
    use crate::TypeLayout;

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
        type Head: TypeLayout;
        type Tail: TypeHList;

        type Concat<L: TypeHList>: TypeHList;
    }

    impl TypeHList for Empty {
        type Concat<L: TypeHList> = L;
        // Empty's head can be anything since we never use it
        type Head = ();
        type Tail = Self;
    }

    impl<H: TypeLayout, T: TypeHList> TypeHList for Cons<H, T> {
        type Concat<L: TypeHList> = Cons<H, T::Concat<L>>;
        type Head = H;
        type Tail = T;
    }
}
