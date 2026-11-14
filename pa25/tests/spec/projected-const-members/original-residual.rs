#![feature(const_trait_impl,const_try,const_try_residual,try_trait_v2,try_trait_v2_residual)]
use core::ops::{Residual,FromResidual};
pub const fn residual_into_try_type<R: [const] Residual<O>, O>(
    r: R,
) -> <R as Residual<O>>::TryType {
    FromResidual::from_residual(r)
}
fn main(){}
