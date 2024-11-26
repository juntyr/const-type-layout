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
    expand_type_layout_stack_graph_into_static_slice::<T, private::Empty, T, private::Empty>(None)
}

// compute the type graph by keeping an on-stack linked list of already-seen
//  types
// as the stack graph is computed, accumulate the hlist type that is layout-
//  equivalent to an array storing all of the type layouts
// once the graph is computed on the stack, recompute it inside a const using
//  the now-known type of the hlist
// a static slice to the array of type layouts is returned
const fn expand_type_layout_stack_graph_into_static_slice<
    GraphRoot: TypeLayout,
    GraphNodesArray: 'static + Copy + Freeze + private::HList,
    T: TypeLayout,
    RemainingTypes: private::TypeHList,
>(
    tys: Option<&GraphStackNode>,
) -> &'static [&'static TypeLayoutInfo<'static>] {
    let layout = &T::TYPE_LAYOUT;

    // check if this type has already been inserted into the graph
    let mut it = &tys;
    while let Some(i) = it {
        if type_layout_info_eq(i.ty, layout) {
            // if no types remain, continue by computing the graph again, this
            //  time in a const with the now-known array hlist type
            if RemainingTypes::LEN == 0 {
                return compute_type_layout_graph_static_slice::<GraphRoot, GraphNodesArray>();
            }

            // if more types remain, continue with recursion
            return expand_type_layout_stack_graph_into_static_slice::<
                GraphRoot,
                GraphNodesArray,
                RemainingTypes::Head,
                RemainingTypes::Tail,
            >(tys);
        }

        it = &i.next;
    }

    if <<T::TypeGraphEdges as private::TypeHList>::Concat<RemainingTypes> as private::HList>::LEN
        == 0
    {
        // if no types remain, also considering the links from T, continue by
        //  computing the graph again, this time in a const with the now-known
        //  array hlist type
        // the GraphNodesArray HList is extended by one element for T
        return compute_type_layout_graph_static_slice::<
            GraphRoot,
            private::Cons<&'static TypeLayoutInfo<'static>, GraphNodesArray>,
        >();
    }

    // if more types remain, continue with recursion, adding the links from T
    //  to the stack of remaining types
    // the GraphNodesArray HList is extended by one element for T
    expand_type_layout_stack_graph_into_static_slice::<
        GraphRoot,
        private::Cons<&'static TypeLayoutInfo<'static>, GraphNodesArray>,
        <<T::TypeGraphEdges as private::TypeHList>::Concat<RemainingTypes> as private::TypeHList>::Head,
        <<T::TypeGraphEdges as private::TypeHList>::Concat<RemainingTypes> as private::TypeHList>::Tail,
    >(Some(&GraphStackNode {
        ty: layout,
        next: tys,
    }))
}

// on-stack linked list node that stores one type layout
struct GraphStackNode<'a> {
    ty: &'static TypeLayoutInfo<'static>,
    next: Option<&'a Self>,
}

// transform the static reference to the hlist that's layout-equivalent to the
//  array storing all type layouts into a static slice to the same array
const fn compute_type_layout_graph_static_slice<
    GraphRoot: TypeLayout,
    GraphNodesArray: 'static + Copy + Freeze + private::HList,
>() -> &'static [&'static TypeLayoutInfo<'static>] {
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
            core::ptr::from_ref(compute_type_layout_graph_static_array_ref::<
                GraphRoot,
                GraphNodesArray,
            >())
            .cast(),
            GraphNodesArray::LEN,
        )
    }
}

// create a static reference to the type layout hlist using an inline const
const fn compute_type_layout_graph_static_array_ref<
    GraphRoot: TypeLayout,
    GraphNodesArray: 'static + Copy + Freeze + private::HList,
>() -> &'static GraphNodesArray {
    &const { compute_type_layout_graph_array::<GraphRoot, GraphNodesArray>() }
}

// create the hlist that's layout-equivalent to the array storing all type
//  layouts and fill it with these type layouts
const fn compute_type_layout_graph_array<GraphRoot: TypeLayout, GraphNodesArray: private::HList>(
) -> GraphNodesArray {
    // layout-compatible with MaybeUninit::<[_; LEN]>::uninit()
    let mut graph = MaybeUninit::<GraphNodesArray>::uninit();

    // SAFETY:
    // - HList is a sealed trait and is constructed here to be made of only Cons of
    //   TypeLayoutInfo and Empty
    // - Cons is a repr(C) struct with a head followed by a tail, Empty is a
    //   zero-sized repr(C) struct
    // - the HList is layout-equivalent to an array of the same length as HList::LEN
    // - the mutable slice is to a slice of uninit elements, and an uninit array can
    //   always be accessed as an array of uninit elements
    let graph_slice = unsafe {
        core::slice::from_raw_parts_mut(
            core::ptr::from_mut(&mut graph).cast(),
            GraphNodesArray::LEN,
        )
    };
    let graph_len = fill_type_layout_graph_slice::<GraphRoot, private::Empty>(graph_slice, 0);

    assert!(
        graph_len == GraphNodesArray::LEN,
        "bug: initialized graph has the wrong size"
    );

    // Safety: we have just checked that all array elements have been initialized
    unsafe { graph.assume_init() }
}

// compute the type graph by filling a slice of uninitialized type layouts and
//  using it to check which types have already been seen
// once the graph is computed, return the number of elements initialized
const fn fill_type_layout_graph_slice<T: TypeLayout, Remaining: private::TypeHList>(
    tys: &mut [MaybeUninit<&'static TypeLayoutInfo<'static>>],
    init_tys_len: usize,
) -> usize {
    let layout = &T::TYPE_LAYOUT;

    // check if this type has already been inserted into the graph
    let mut i = 0;
    while i < init_tys_len {
        // Safety: tys has been initialized for 0..tys_len
        if type_layout_info_eq(unsafe { tys[i].assume_init_ref() }, layout) {
            // if no types remain, return the initialized length of the array
            if Remaining::LEN == 0 {
                return init_tys_len;
            }

            // if more types remain, continue with recursion
            return fill_type_layout_graph_slice::<Remaining::Head, Remaining::Tail>(
                tys,
                init_tys_len,
            );
        }
        i += 1;
    }

    // push the type layout into the slice to initialize the next element
    assert!(init_tys_len < tys.len(), "bug: type layout graph too small");
    tys[init_tys_len] = MaybeUninit::new(layout);

    if <<T::TypeGraphEdges as private::TypeHList>::Concat<Remaining> as private::HList>::LEN == 0 {
        // if no types remain, also considering the links from T, return the
        //  initialized length of the array
        return init_tys_len + 1;
    }

    // if more types remain, continue with recursion, adding the links from T
    //  to the stack of remaining types
    fill_type_layout_graph_slice::<
        <<T::TypeGraphEdges as private::TypeHList>::Concat<Remaining> as private::TypeHList>::Head,
        <<T::TypeGraphEdges as private::TypeHList>::Concat<Remaining> as private::TypeHList>::Tail,
    >(tys, init_tys_len + 1)
}

// simple type layout info equality by checking the equality of their type names
const fn type_layout_info_eq(a: &TypeLayoutInfo, b: &TypeLayoutInfo) -> bool {
    str_eq(a.name, b.name)
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
