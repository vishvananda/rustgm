#[lang="freeze"] pub unsafe auto trait Freeze {}
#[cfg(rustc_oracle)] #[lang="drop_glue"] pub unsafe fn drop_glue<T:PointeeSized>(_:&mut T) {}
impl Clone for bool {fn clone(&self)->Self{*self}} impl Copy for bool {}
impl Clone for u32 {fn clone(&self)->Self{*self}} impl Copy for u32 {}
impl<T:PointeeSized> Clone for &T {fn clone(&self)->Self{*self}} impl<T:PointeeSized> Copy for &T {}
#[lang="deref"] pub const trait Deref: PointeeSized {
    #[lang="deref_target"] type Target:?Sized;
    fn deref(&self)->&Self::Target;
}
pub struct Value(pub u32);
pub struct Outer(pub Value);
pub struct Double(pub Outer);
impl Deref for Outer {
    type Target=Value;
    fn deref(&self)->&Value { &self.0 }
}
impl Deref for Double {
    type Target=Outer;
    fn deref(&self)->&Outer { &self.0 }
}
impl Value {
    pub fn choose(&self, other:&Value, first:bool)->u32 {
        if first {self.0} else {other.0}
    }
}
pub trait Select { fn select(&self, other:&Value, first:bool)->u32; }
impl Select for Value {
    fn select(&self, other:&Value, first:bool)->u32 { self.choose(other,first) }
}
pub fn direct(x:&Value, other:&Value, first:bool)->u32 { x.choose(other,first) }
pub fn through(x:&Outer, other:&Value, first:bool)->u32 { x.choose(other,first) }
pub fn through_nested(x:&Double, other:&Value, first:bool)->u32 { x.choose(other,first) }
pub fn through_generic<D:Deref<Target=Value>>(x:&D, other:&Value, first:bool)->u32 {
    x.choose(other,first)
}
pub fn through_trait(x:&Outer, other:&Value, first:bool)->u32 { x.select(other,first) }
