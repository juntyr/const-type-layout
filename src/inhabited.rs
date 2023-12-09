#[allow(clippy::empty_enum)]
pub enum Inhabited {}

#[allow(clippy::empty_enum)]
pub enum Uninhabited {}

#[allow(clippy::module_name_repetitions)]
pub trait ComputeInhabited: sealed::ComputeInhabited {
    type Output: OutputMaybeInhabited;
}

#[allow(clippy::module_name_repetitions)]
pub trait OutputMaybeInhabited: ComputeInhabited + sealed::OutputMaybeInhabited {}

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

pub macro all {
    () => { Inhabited },
    ($L:ty $(, $R:ty)*) => {
        <And<<$L as $crate::TypeLayout>::Inhabited, all![$($R),*]> as ComputeInhabited>::Output
    },
}

pub macro any {
    () => { Uninhabited },
    ($L:ty $(, $R:ty)*) => {
        <Or<<$L as $crate::TypeLayout>::Inhabited, any![$($R),*]> as ComputeInhabited>::Output
    },
}
