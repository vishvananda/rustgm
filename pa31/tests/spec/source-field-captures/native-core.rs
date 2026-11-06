use core::ops::{Deref,DerefMut};
// Authored capture consumer: unchanged between source-provider and standard-core oracle.
unsafe extern "C"{fn record(value:u8);}
pub struct Leaf{pub value:u8}
pub struct Inner{pub leaf:Leaf}
pub struct Wrap{pub inner:Inner,pub sibling:u8}
impl Deref for Wrap{type Target=Inner;fn deref(&self)->&Inner{unsafe{record(1)};&self.inner}}
impl DerefMut for Wrap{fn deref_mut(&mut self)->&mut Inner{unsafe{record(2)};&mut self.inner}}
impl Drop for Wrap{fn drop(&mut self){unsafe{record(9)}}}
fn make()->Wrap{Wrap{inner:Inner{leaf:Leaf{value:13}},sibling:7}}
fn temporary()->Wrap{unsafe{record(3)};make()}
fn want_fn<F:Fn()->u8>(f:&F)->u8{f()}
fn want_mut<F:FnMut()->u8>(f:&mut F)->u8{f()}
fn want_once<F:FnOnce()->u8>(f:F)->u8{f()}
fn take(_:Wrap){}
#[unsafe(no_mangle)]pub extern "C" fn cap_shared()->u8{let x=make();let f=||x.leaf.value;unsafe{record(10)};let _=want_fn(&f);unsafe{record(11)};let v=want_fn(&f);unsafe{record(12)};v}
#[unsafe(no_mangle)]pub extern "C" fn cap_mutable()->u8{let mut x=make();let v={let mut f=||{x.leaf.value=23;x.inner.leaf.value};unsafe{record(20)};let _=want_mut(&mut f);unsafe{record(21)};let v=want_mut(&mut f);unsafe{record(22)};v};v}
#[unsafe(no_mangle)]pub extern "C" fn cap_move_read()->u8{let f={let x=make();move||x.leaf.value};unsafe{record(30)};let _=want_fn(&f);unsafe{record(31)};let v=want_fn(&f);unsafe{record(32)};v}
#[unsafe(no_mangle)]pub extern "C" fn cap_once()->u8{let f={let x=make();move||{let v=x.leaf.value;take(x);v}};unsafe{record(40)};let v=want_once(f);unsafe{record(41)};v}
#[unsafe(no_mangle)]pub extern "C" fn cap_unpolled(){let _f={let x=make();move||x.leaf.value};unsafe{record(50)}}
pub struct Owner{pub left:Wrap,pub right:u8}
#[unsafe(no_mangle)]pub extern "C" fn cap_disjoint()->u8{let mut x=Owner{left:make(),right:7};let f=||x.left.leaf.value;x.right=41;unsafe{record(60)};let v=f();unsafe{record(x.right)};v}
#[unsafe(no_mangle)]pub extern "C" fn cap_temporary()->u8{let f=||temporary().leaf.value;unsafe{record(70)};let _=f();unsafe{record(71)};let v=f();unsafe{record(72)};v}
#[unsafe(no_mangle)]pub extern "C" fn cap_nested()->u8{let x=make();let f=||{let g=||x.leaf.value;unsafe{record(81)};g()};unsafe{record(80)};let _=f();unsafe{record(82)};let v=f();unsafe{record(83)};v}
pub struct LeafR<'a>{pub item:&'a u8}pub struct InnerR<'a>{pub leaf:LeafR<'a>}pub struct WrapR<'a>{pub inner:InnerR<'a>}
impl<'a>Deref for WrapR<'a>{type Target=InnerR<'a>;fn deref(&self)->&InnerR<'a>{&self.inner}}
fn field_ref<'a>(x:&'a WrapR<'a>)->impl Fn()->&'a u8+'a{move||x.leaf.item}
#[unsafe(no_mangle)]pub extern "C" fn cap_reference()->u8{let value=67;let x=WrapR{inner:InnerR{leaf:LeafR{item:&value}}};let f=field_ref(&x);*f()}
