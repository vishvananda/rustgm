#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

pub fn shorten<'a: 'b, 'b>(x: &'a u8, _y: &'b u8) -> &'b u8 {
    x
}

pub fn pick_early<'late, 'early, T: 'early>(_x: &'late u8, y: &'early T) -> &'early T {
    y
}

pub fn unused<'a>(x: u8) -> u8 {
    x
}

pub fn unused_generic<'a, T>(x: T) -> T {
    x
}

pub struct Held<'a> {
    pub value: &'a u8,
}

impl<'a> Held<'a> {
    pub const fn shorter<'b>(&self, _witness: &'b u8) -> &'b u8
    where
        'a: 'b,
    {
        self.value
    }

    pub fn with_early<'late, 'b>(&self, _other: &'late u8, _witness: &'b u8) -> &'b u8
    where
        'a: 'b,
    {
        self.value
    }
}

pub trait Lend<'a> {
    fn lend<'b>(&self, witness: &'b u8) -> &'b u8
    where
        'a: 'b;
}

impl<'a> Lend<'a> for Held<'a> {
    fn lend<'b>(&self, _witness: &'b u8) -> &'b u8
    where
        'a: 'b,
    {
        self.value
    }
}

pub fn call_lend<'a: 'b, 'b, L: Lend<'a>>(l: &L, witness: &'b u8) -> &'b u8 {
    l.lend(witness)
}

pub fn via_provider<'a: 'b, 'b>(x: &'a u8, y: &'b u8) -> (u8, u8, u8, u8) {
    let held = Held { value: &*x };
    let a = shorten::<'a, 'b>(x, &*y);
    let b = held.with_early(&*y, &*y);
    let c = pick_early(y, &*a);
    let d = unused_generic(1u8);
    (*a, *b, *c, unused(d))
}
