#[doc(hidden)]
pub trait ComputeSet: sealed::ComputeSet {
    const LEN: usize;

    type Output<H: ComputeTypeSet>: ExpandTypeSet;

    type TyHList: 'static + Copy;
    const TYS: &'static Self::TyHList;
}

mod sealed {
    pub trait ComputeSet {}

    impl ComputeSet for super::private::Empty {}
    impl<H2: super::ComputeTypeSet, T: ComputeSet> ComputeSet for super::private::Cons<H2, T> {}
}

pub type Set<H, T> = <T as ComputeSet>::Output<H>;

/// # Safety
///
/// It is only safe to implement this trait if it accurately includes
/// all type components that are referenced by this type's layout. Use
/// `#[derive(TypeLayout)]` instead.
pub unsafe trait ComputeTypeSet: crate::TypeLayout {
    type Output<T: ExpandTypeSet>: ExpandTypeSet;
}

pub macro tset {
    () => { Empty },
    (.. @ $T:tt) => { $T },
    ($H:ty, $($R:ty,)*) => {
        Set<$H, tset![$($R,)*]>
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
    type Output<R: ExpandTypeSet> = <T as ExpandTypeSet>::Output<<H as ComputeTypeSet>::Output<R>>;
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
        type Output<H: ComputeTypeSet> = Cons<H, Empty>;
        type TyHList = Empty;

        const LEN: usize = 0;
        const TYS: &'static Self::TyHList = &Empty;
    }

    impl<H2: ComputeTypeSet, T: ExpandTypeSet> ComputeSet for Cons<H2, T> {
        type Output<H1: ComputeTypeSet> = <Cons<H2, T> as ComputeCons<H1>>::Output;
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
        type Output = Cons<H, Empty>;
    }

    impl<H: ComputeTypeSet, T: ExpandTypeSet> ComputeCons<H> for Cons<H, T> {
        type Output = Cons<H, T>;
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

pub type TypeSet<T> = <<T as ComputeTypeSet>::Output<private::Empty> as TypeSetFixedPoint>::Output;
