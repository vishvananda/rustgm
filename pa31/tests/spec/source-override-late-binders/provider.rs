#[lang="freeze"] pub unsafe auto trait Freeze {}
#[cfg(rustc_oracle)]
#[lang="drop_glue"] pub unsafe fn drop_glue<T:PointeeSized>(_:&mut T) {}
impl Clone for bool {fn clone(&self)->Self{*self}} impl Copy for bool {}
impl Clone for u32 {fn clone(&self)->Self{*self}} impl Copy for u32 {}
impl<T:PointeeSized> Clone for &T {fn clone(&self)->Self{*self}} impl<T:PointeeSized> Copy for &T {}
#[lang="eq"]
pub const trait PartialEq<Rhs: PointeeSized=Self>: PointeeSized {
    fn eq(&self, rhs:&Rhs)->bool;
    fn ne(&self, rhs:&Rhs)->bool { if self.eq(rhs) { false } else { true } }
}
const impl PartialEq for u32 {
    fn eq(&self, rhs:&u32)->bool { *self == *rhs }
}
impl<A:?Sized,B:?Sized> PartialEq<&B> for &A where A:PartialEq<B> {
    fn eq(&self,rhs:&&B)->bool { <A as PartialEq<B>>::eq(*self,*rhs) }
}
pub struct Wrap(pub u32);
impl PartialEq for Wrap {
    fn eq(&self,rhs:&Self)->bool { &self.0 == &rhs.0 }
}
pub fn compare(x:&Wrap,y:&Wrap)->bool { x.eq(y) }

pub fn references(x:&u32,y:&u32)->bool { x == y }
pub fn nested(x:&&u32,y:&&u32)->bool { x == y }
pub fn generic<T:PartialEq>(x:&T,y:&T)->bool { x == y }
pub fn forwarded(x:u32,y:u32)->bool { generic(&Wrap(x),&Wrap(y)) }
