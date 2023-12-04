#[doc(hidden)]
pub trait ComputeSet: sealed::ComputeSet {
    type Output<H>: ComputeSet;
}

mod sealed {
    pub trait ComputeSet {}

    impl ComputeSet for super::Empty {}
    impl<H2, T: ComputeSet> ComputeSet for super::Cons<H2, T> {}
}

#[allow(clippy::empty_enum)]
pub enum Empty {}

pub struct Cons<H, T> {
    _head: H,
    _tail: T,
}

impl ComputeSet for Empty {
    type Output<H> = Cons<H, Empty>;
}

impl<H2, T: ComputeSet> ComputeSet for Cons<H2, T> {
    type Output<H1> = <Cons<H2, T> as ComputeCons<H1>>::Output;
}

#[doc(hidden)]
pub trait ComputeCons<H>: sealed::ComputeSet {
    type Output: ComputeSet;
}

impl<H> ComputeCons<H> for Empty {
    type Output = Cons<H, Empty>;
}

impl<H, T: ComputeSet> ComputeCons<H> for Cons<H, T> {
    type Output = Cons<H, T>;
}

impl<H1, H2, T: ComputeSet> ComputeCons<H1> for Cons<H2, T> {
    default type Output = Cons<H2, Set<H1, T>>;
}

pub type Set<H, T> = <T as ComputeSet>::Output<H>;

/// # Safety
///
/// It is only safe to implement this trait if it accurately includes
/// all type components that are referenced by this type's layout.
pub unsafe trait ComputeTypeSet {
    type Output<T: ComputeSet>: ComputeSet;
}

pub macro tset {
    ([$H:ty] => $T:ty) => {
        <$H as ComputeTypeSet>::Output::<$T>
    },
    ([$H:ty, $($R:ty),*] => $T:ty) => {
        <$H as ComputeTypeSet>::Output::<
            tset!([$($R),*] => $T)
        >
    },
}

pub type TypeSet<T> = <T as ComputeTypeSet>::Output<Empty>;
