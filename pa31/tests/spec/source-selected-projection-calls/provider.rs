#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for &T {}

pub trait Read { type Out; fn read(&self) -> Self::Out; }
pub trait Borrow { type Out; fn borrow(&self) -> &Self::Out; }
pub struct Cell(pub u32);
pub struct Other(pub u32);
impl Read for Cell {
    type Out = u32;
    fn read(&self) -> Self::Out { self.0 }
}
impl Read for Other {
    type Out = u32;
    fn read(&self) -> Self::Out { self.0 }
}
impl Borrow for Cell {
    type Out = u32;
    fn borrow(&self) -> &Self::Out { &self.0 }
}
pub fn direct(x: u32) -> u32 { Cell(x).read() }
pub fn other(x: u32) -> u32 { Other(x).read() }
pub fn borrowed(x: &Cell) -> &u32 { x.borrow() }
pub fn generic<T: Read<Out = u32>>(x: &T) -> u32 { x.read() }
pub fn forwarded(x: u32) -> u32 { generic(&Cell(x)) }

#[lang = "index"]
pub const trait Index<I: ?Sized> {
    type Output: ?Sized;
    fn index(&self, index: I) -> &Self::Output;
}
#[lang = "index_mut"]
pub const trait IndexMut<I: ?Sized>: [const] Index<I> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output;
}
pub struct Pair(pub u32, pub u32);
pub trait Select {
    type Out: ?Sized;
    fn shared<'a>(&self, pair: &'a Pair) -> &'a Self::Out;
    fn exclusive<'a>(&self, pair: &'a mut Pair) -> &'a mut Self::Out;
}
impl Select for bool {
    type Out = u32;
    fn shared<'a>(&self, pair: &'a Pair) -> &'a u32 {
        if *self { &pair.1 } else { &pair.0 }
    }
    fn exclusive<'a>(&self, pair: &'a mut Pair) -> &'a mut u32 {
        if *self { &mut pair.1 } else { &mut pair.0 }
    }
}
impl<I: Select> Index<I> for Pair {
    type Output = I::Out;
    fn index(&self, index: I) -> &Self::Output { index.shared(self) }
}
impl<I: Select> IndexMut<I> for Pair {
    fn index_mut(&mut self, index: I) -> &mut Self::Output { index.exclusive(self) }
}
pub fn indexed(x: &Pair, flag: bool) -> u32 { x[flag] }
pub fn indexed_mut(x: &mut Pair, value: u32) -> u32 {
    x[true] = value;
    x[true]
}

#[cfg(not(older_profile))]
#[lang = "deref"]
pub const trait Deref: PointeeSized {
    #[lang = "deref_target"]
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}
#[cfg(not(older_profile))]
pub struct Wrapper<I>(pub I, pub Pair);
#[cfg(not(older_profile))]
impl<I: Select> Deref for Wrapper<I> {
    type Target = I::Out;
    fn deref(&self) -> &Self::Target { self.0.shared(&self.1) }
}
#[cfg(not(older_profile))]
pub fn implicit(x: &Wrapper<bool>) -> u32 {
    let value: &u32 = x;
    *value
}
