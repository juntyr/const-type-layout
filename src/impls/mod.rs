use crate::TypeLayout;

mod alloc;
mod core;

const unsafe fn leak_uninit_ptr<T: ~const TypeLayout>() -> *mut T {
    const fn alloc_comptime<T>() -> *mut T {
        unsafe {
            ::core::intrinsics::const_allocate(
                ::core::mem::size_of::<T>(),
                ::core::mem::align_of::<T>(),
            )
        }
        .cast()
    }

    fn alloc_runtime<T>() -> *mut T {
        unsafe { ::alloc::alloc::alloc(::alloc::alloc::Layout::new::<T>()) }.cast()
    }

    let ptr = ::core::intrinsics::const_eval_select((), alloc_comptime, alloc_runtime);

    ::core::ptr::write(ptr, <T as TypeLayout>::uninit().assume_init());

    ptr
}
