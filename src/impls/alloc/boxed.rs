use alloc::alloc::Global;

use crate::{TypeGraph, TypeLayout, TypeLayoutGraph, TypeLayoutInfo, TypeStructure};

unsafe impl<T: TypeLayout> TypeLayout for alloc::boxed::Box<T> {
    type Static = alloc::boxed::Box<T::Static>;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Pointer {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new(unsafe {
        alloc::boxed::Box::from_raw_in(
            <Self as BoxElem<T::Static>>::ELEM as *const T::Static as *mut T,
            alloc::alloc::Global,
        )
    });
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for alloc::boxed::Box<T> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}

trait BoxElem<T: 'static> {
    const ELEM: &'static T;
}

impl<T: TypeLayout> BoxElem<T::Static> for alloc::boxed::Box<T> {
    const ELEM: &'static T::Static = unsafe {
        let ptr: *mut T =
            core::intrinsics::const_allocate(core::mem::size_of::<T>(), core::mem::align_of::<T>())
                .cast();

        core::ptr::write(
            ptr,
            core::mem::ManuallyDrop::into_inner(<T as TypeLayout>::UNINIT),
        );

        &*ptr.cast()
    };
}

unsafe impl<T: TypeLayout> TypeLayout for alloc::boxed::Box<[T]> {
    type Static = alloc::boxed::Box<[T::Static]>;

    const TYPE_LAYOUT: TypeLayoutInfo<'static> = TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: TypeStructure::Pointer {
            inner: ::core::any::type_name::<T>(),
            mutability: true,
        },
    };
    #[allow(clippy::borrow_as_ptr)]
    const UNINIT: core::mem::ManuallyDrop<Self> = core::mem::ManuallyDrop::new(unsafe {
        alloc::boxed::Box::from_raw_in(&[] as *const [T] as *mut _, Global)
    });
}

unsafe impl<T: ~const TypeGraph> const TypeGraph for alloc::boxed::Box<[T]> {
    fn populate_graph(graph: &mut TypeLayoutGraph<'static>) {
        if graph.insert(&Self::TYPE_LAYOUT) {
            <T as TypeGraph>::populate_graph(graph);
        }
    }
}
