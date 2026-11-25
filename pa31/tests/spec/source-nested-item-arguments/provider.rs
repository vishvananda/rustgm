#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
#[cfg(not(old_nested_items))]
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
#[cfg(not(old_nested_items))]
impl<T: PointeeSized> Copy for &T {}

fn one(x: u32) -> u32 { match x { 5 => 11, _ => x } }
fn two(x: u32) -> u32 { match x { 5 => 22, _ => x } }
fn identity(x: &u32) -> &u32 { x }

fn pass<T>(x: T) -> T { x }

#[cfg(not(old_nested_items))]
struct Holder<T>(T);

#[cfg(not(old_nested_items))]
pub fn tuple_items() -> u32 {
    let (f, x) = pass((one, 5u32));
    f(x)
}

#[cfg(not(old_nested_items))]
pub fn nominal_items() -> u32 {
    let Holder(f) = pass(Holder(two));
    f(5)
}

#[cfg(not(old_nested_items))]
pub fn array_items() -> u32 {
    let [f, _] = pass([one, one]);
    f(5)
}

#[cfg(not(old_nested_items))]
pub fn reference_items() -> u32 {
    let f = pass(&two);
    f(5)
}

#[cfg(not(old_nested_items))]
pub fn choose_refs<'a>(x: &'a u32, y: &'a u32, first: bool) -> &'a u32 {
    let (f,) = pass((identity,));
    if first { f(x) } else { f(y) }
}

#[cfg(not(old_nested_items))]
pub fn tuple_second() -> u32 {
    let (f, x) = pass((two, 5u32));
    f(x)
}

// Keep the older-profile control focused on the argument recipe, without
// the newer nominal projections or applied reference method frames.
#[cfg(old_nested_items)]
pub fn older_profile_probe() -> u32 {
    let _ = pass((one,));
    11
}
