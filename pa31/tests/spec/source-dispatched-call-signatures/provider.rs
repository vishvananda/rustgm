#[lang="freeze"] pub unsafe auto trait Freeze {}
#[cfg(rustc_oracle)] #[lang="drop_glue"] pub unsafe fn drop_glue<T:PointeeSized>(_:&mut T) {}
impl Clone for u32 {fn clone(&self)->Self{*self}} impl Copy for u32 {}
pub trait Convert {fn convert(x:u8)->u32;}
impl Convert for u32 {fn convert(x:u8)->u32{x as u32}}
pub struct Other;
impl Convert for Other {fn convert(x:u8)->u32{match x {11=>99,_=>x as u32}}}
pub fn apply<F:FnOnce(u8)->u32>(f:F,x:u8)->u32 {f(x)}
pub fn through(x:u8)->u32 {apply(u32::convert,x)}
pub struct Wrap<T>(pub T);
impl<T> Wrap<T> {
    pub fn map<U,F:FnOnce(T)->U>(self,f:F)->Wrap<U> {Wrap(f(self.0))}
}
pub fn mapped(x:u8)->u32 {Wrap(x).map(u32::convert).0}
fn pass<T>(x:T)->T{x}
pub fn tuple(x:u8)->u32 {let (f,x)=pass((u32::convert,x));f(x)}
pub fn nominal(x:u8)->u32 {let Wrap(f)=pass(Wrap(u32::convert));f(x)}
pub fn array(x:u8)->u32 {let [f,_]=pass([u32::convert,u32::convert]);f(x)}
pub fn distinct(x:u8)->u32 {apply(Other::convert,x)}
pub fn forward<F:FnOnce(u8)->u32>(f:F,x:u8)->u32 {apply(f,x)}
pub fn forwarded(x:u8)->u32 {forward(u32::convert,x)}
pub unsafe fn unsafe_apply<F:FnOnce(u8)->u32>(f:F,x:u8)->u32 {f(x)}
pub fn explicit_unsafe(x:u8)->u32 {unsafe {unsafe_apply(u32::convert,x)}}
pub unsafe fn unsafe_scope(x:u8)->u32 {unsafe_apply(u32::convert,x)}
