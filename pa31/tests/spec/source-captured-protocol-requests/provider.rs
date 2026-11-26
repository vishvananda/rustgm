#[lang="freeze"] pub unsafe auto trait Freeze {}
#[cfg(rustc_oracle)] #[lang="drop_glue"] pub unsafe fn drop_glue<T:PointeeSized>(_:&mut T) {}
impl Clone for u32 {fn clone(&self)->Self{*self}} impl Copy for u32 {}
impl Clone for usize {fn clone(&self)->Self{*self}} impl Copy for usize {}
impl<T:PointeeSized> Clone for &T {fn clone(&self)->Self{*self}} impl<T:PointeeSized> Copy for &T {}
#[lang="deref"] pub const trait Deref:PointeeSized {
    #[lang="deref_target"] type Target:?Sized;
    fn deref(&self)->&Self::Target;
}
#[lang="deref_mut"] pub const trait DerefMut:[const] Deref+PointeeSized {
    fn deref_mut(&mut self)->&mut Self::Target;
}
#[lang="index"] pub const trait Index<I:?Sized> {
    type Output:?Sized;
    fn index(&self,index:I)->&Self::Output;
}
pub struct Inner<'a> {pub value:&'a u32}
pub struct Outer<'a>(pub Inner<'a>);
pub struct Double<'a>(pub Outer<'a>);
impl<'a> Inner<'a> {
    pub fn read(&self)->u32 {*self.value}
    pub fn borrow(&self)->&'a u32 {self.value}
    pub fn replace(&mut self,next:&'a u32)->&'a u32 {
        let old=self.value;self.value=next;old
    }
}
impl<'a> Deref for Outer<'a> {
    type Target=Inner<'a>;
    fn deref(&self)->&Inner<'a> {&self.0}
}
impl<'a> DerefMut for Outer<'a> {
    fn deref_mut(&mut self)->&mut Inner<'a> {&mut self.0}
}
impl<'a> Deref for Double<'a> {
    type Target=Outer<'a>;
    fn deref(&self)->&Outer<'a> {&self.0}
}
impl<'a> Index<usize> for Inner<'a> {
    type Output=u32;
    fn index(&self,_:usize)->&u32 {self.value}
}
pub fn through<'a>(x:&Outer<'a>)->u32 {x.read()}
pub fn borrowed<'a>(x:&Outer<'a>)->&'a u32 {x.borrow()}
pub fn nested<'a>(x:&Double<'a>)->u32 {x.read()}
pub fn nested_borrow<'a>(x:&Double<'a>)->&'a u32 {x.borrow()}
pub fn coerced<'a>(x:&Outer<'a>)->u32 {let inner:&Inner<'a>=x;inner.read()}
pub fn field<'a>(x:&Outer<'a>)->u32 {*x.value}
pub fn indexed<'a>(x:&Outer<'a>)->u32 {x[0]}
pub fn direct_index<'a>(x:&Inner<'a>)->u32 {x[0]}
pub fn replaced<'a>(x:&mut Outer<'a>,next:&'a u32)->u32 {*x.replace(next)}
pub fn control<'a>(x:&Inner<'a>)->u32 {x.read()}
