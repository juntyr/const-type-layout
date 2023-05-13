use crate::{MaybeUninhabited, TypeLayout};

mod core;

#[doc(hidden)]
pub const unsafe fn leak_uninit_ptr<T: ~const TypeLayout>() -> *mut T {
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

    if let MaybeUninhabited::Inhabited(uninit) = <T as TypeLayout>::uninit() {
        ::core::ptr::write(ptr, uninit.assume_init());
    }

    ptr
}
