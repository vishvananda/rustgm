#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for &T {}

pub trait View<'a> {
    type Out: 'a;
    fn view(&'a self) -> Self::Out;
}

pub trait Double<'a, 'b> {
    type Out;
    fn combine(&'a self, other: &'b u32) -> Self::Out;
}

pub struct Cell(pub u32);
pub struct Plain(pub u32);

impl<'a> View<'a> for Cell {
    type Out = &'a u32;
    fn view(&'a self) -> &'a u32 {
        &self.0
    }
}

impl<'a, 'b> Double<'a, 'b> for Cell {
    type Out = (&'a u32, &'b u32);
    fn combine(&'a self, other: &'b u32) -> (&'a u32, &'b u32) {
        (&self.0, other)
    }
}

fn view_step<'a, T>(v: &'a T) -> &'a u32
where
    for<'x> T: View<'x, Out = &'x u32>,
{
    <T as View<'a>>::view(v)
}

fn double_step<'a, 'b, T>(v: &'a T, other: &'b u32) -> (&'a u32, &'b u32)
where
    for<'x, 'y> T: Double<'x, 'y, Out = (&'x u32, &'y u32)>,
{
    <T as Double<'a, 'b>>::combine(v, other)
}

pub fn apply_hrtb<T>(v: &T, other: &u32) -> (u32, u32, u32)
where
    for<'a> T: View<'a, Out = &'a u32>,
    for<'a, 'b> T: Double<'a, 'b, Out = (&'a u32, &'b u32)>,
{
    let f: for<'a> fn(&'a T) -> <T as View<'a>>::Out = view_step::<T>;
    let g: for<'a, 'b> fn(&'a T, &'b u32) -> <T as Double<'a, 'b>>::Out = double_step::<T>;
    let a = f(v);
    let (b, c) = g(v, other);
    (*a, *b, *c)
}

pub fn call_higher_ranked<F>(f: F, x: &u32, y: &u32) -> (u32, u32)
where
    F: for<'a> Fn(&'a u32) -> &'a u32,
{
    (*f(x), *f(y))
}

pub fn nested_hrtb<T>(v: &T, other: &u32) -> (u32, u32, u32)
where
    for<'a> T: View<'a, Out = &'a u32>,
    for<'a, 'b> T: Double<'a, 'b, Out = (&'a u32, &'b u32)>,
{
    let pair: (
        for<'a> fn(&'a T) -> <T as View<'a>>::Out,
        for<'a, 'b> fn(&'a T, &'b u32) -> <T as Double<'a, 'b>>::Out,
    ) = (view_step::<T>, double_step::<T>);
    let a = (pair.0)(v);
    let (b, c) = (pair.1)(v, other);
    (*a, *b, *c)
}

pub const fn const_hrtb_passthrough<T>(seed: u32) -> u32
where
    for<'a> T: View<'a, Out = &'a u32>,
{
    let _f: for<'a> fn(&'a T) -> <T as View<'a>>::Out = view_step::<T>;
    seed
}

pub fn id_ref<'a>(x: &'a u32) -> &'a u32 {
    x
}
