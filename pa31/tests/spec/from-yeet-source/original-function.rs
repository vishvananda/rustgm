#[unstable(
    feature = "yeet_desugar_details",
    issue = "none",
    reason = "just here to simplify the desugaring; will never be stabilized"
)]
#[inline]
#[track_caller] // because `Result::from_residual` has it
#[lang = "from_yeet"]
#[allow(unreachable_pub)] // not-exposed but still used via lang-item
pub fn from_yeet<T, Y>(yeeted: Y) -> T
where
    T: FromResidual<Yeet<Y>>,
{
    FromResidual::from_residual(Yeet(yeeted))
}
