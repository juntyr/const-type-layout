pub struct Inhabited;

unsafe impl crate::TypeLayout for Inhabited {
    type Inhabited = Inhabited;

    const TYPE_LAYOUT: crate::TypeLayoutInfo<'static> = crate::TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: crate::TypeStructure::Struct {
            repr: "",
            fields: &[],
        },
    };
}

unsafe impl crate::typeset::ComputeTypeSet for Inhabited {
    type Output<T: crate::typeset::ExpandTypeSet> = crate::typeset::Set<Self, T>;
}

#[allow(clippy::empty_enum)]
pub enum Uninhabited {}

unsafe impl crate::TypeLayout for Uninhabited {
    type Inhabited = Uninhabited;

    const TYPE_LAYOUT: crate::TypeLayoutInfo<'static> = crate::TypeLayoutInfo {
        name: ::core::any::type_name::<Self>(),
        size: ::core::mem::size_of::<Self>(),
        alignment: ::core::mem::align_of::<Self>(),
        structure: crate::TypeStructure::Enum {
            repr: "",
            variants: &[],
        },
    };
}

unsafe impl crate::typeset::ComputeTypeSet for Uninhabited {
    type Output<T: crate::typeset::ExpandTypeSet> = crate::typeset::Set<Self, T>;
}

#[allow(clippy::module_name_repetitions)]
#[doc(hidden)]
pub trait ComputeInhabited: sealed::ComputeInhabited {
    type Output: OutputMaybeInhabited;
}

#[allow(clippy::module_name_repetitions)]
#[doc(hidden)]
pub trait OutputMaybeInhabited:
    ComputeInhabited + crate::TypeGraphLayout + sealed::OutputMaybeInhabited
{
}

mod sealed {
    pub trait ComputeInhabited {}
    pub trait OutputMaybeInhabited {}
}

impl sealed::ComputeInhabited for Inhabited {}
impl ComputeInhabited for Inhabited {
    type Output = Inhabited;
}
impl sealed::OutputMaybeInhabited for Inhabited {}
impl OutputMaybeInhabited for Inhabited {}

impl sealed::ComputeInhabited for Uninhabited {}
impl ComputeInhabited for Uninhabited {
    type Output = Uninhabited;
}
impl sealed::OutputMaybeInhabited for Uninhabited {}
impl OutputMaybeInhabited for Uninhabited {}

mod logical {
    use super::{sealed, ComputeInhabited, Inhabited, Uninhabited};

    pub struct And<L: ComputeInhabited, R: ComputeInhabited> {
        _left: L,
        _right: R,
    }

    impl<L: ComputeInhabited, R: ComputeInhabited> sealed::ComputeInhabited for And<L, R> {}
    impl<L: ComputeInhabited, R: ComputeInhabited> ComputeInhabited for And<L, R> {
        default type Output = Uninhabited;
    }

    impl<L: ComputeInhabited<Output = Inhabited>, R: ComputeInhabited<Output = Inhabited>>
        ComputeInhabited for And<L, R>
    {
        type Output = Inhabited;
    }

    pub struct Or<L: ComputeInhabited, R: ComputeInhabited> {
        _left: L,
        _right: R,
    }

    impl<L: ComputeInhabited, R: ComputeInhabited> sealed::ComputeInhabited for Or<L, R> {}
    impl<L: ComputeInhabited, R: ComputeInhabited> ComputeInhabited for Or<L, R> {
        default type Output = Inhabited;
    }

    impl<L: ComputeInhabited<Output = Uninhabited>, R: ComputeInhabited<Output = Uninhabited>>
        ComputeInhabited for Or<L, R>
    {
        type Output = Uninhabited;
    }
}

pub macro all {
    () => { Inhabited },
    ($L:ty $(, $R:ty)*) => {
        <logical::And<<$L as $crate::TypeLayout>::Inhabited, all![$($R),*]> as ComputeInhabited>::Output
    },
}

pub macro any {
    () => { Uninhabited },
    ($L:ty $(, $R:ty)*) => {
        <logical::Or<<$L as $crate::TypeLayout>::Inhabited, any![$($R),*]> as ComputeInhabited>::Output
    },
}
