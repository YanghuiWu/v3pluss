/// Type alias for the iteration vector, with i32 elements.
pub type IterVec = Vec<i32>;
/// Type alias for the array access indices, with usize elements.
pub type AryAcc = Vec<usize>;
pub(crate) type DynamicBoundFunction = dyn for<'a> Fn(&'a [i32]) -> i32;
pub(crate) type DynFunc = dyn for<'a> Fn(&'a [i32]) -> Vec<usize>;
