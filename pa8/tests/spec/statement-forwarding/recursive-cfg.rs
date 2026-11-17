macro_rules! check_feature {
    ($feature:tt) => { check_feature!($feature : $feature) };
    ($feature:tt : $($target:tt),*) => { $(cfg!(target_feature = $target);)* };
}
const _: () = { check_feature!("sse2"); };
fn main() {}
