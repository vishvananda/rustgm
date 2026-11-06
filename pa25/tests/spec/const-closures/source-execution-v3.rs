// Authored reducer following pinned core/contracts.rs and callable language traits.
// Authored protocol reducer; interfaces follow pinned core/ops/try_trait.rs and control_flow.rs.
#![feature(unboxed_closures,auto_traits,const_closures,stmt_expr_attributes)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![feature(const_destruct)]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}

#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
#[lang="add"]pub const trait Add<Rhs=Self>{type Output;fn add(self,rhs:Rhs)->Self::Output;}
const impl Add for usize{type Output=Self;fn add(self,rhs:Self)->Self{self+rhs}}
/// Helper function for `drop_in_place`. The compiler replaces this by the actual drop glue.
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

pub const fn folded<F:[const]FnMut(usize,u8)->usize>(mut call:F)->usize
where F:[const]Destruct {
    let first=call(0,1);
    call(first,2)
}
pub const fn counted()->usize {
    folded(#[rustc_inherit_overflow_checks]const |accum,_elem|accum+1)
}
pub const fn captured(value:usize)->usize {
    let call=const move |extra:usize|value+extra;
    call(2)
}
pub const COUNT:usize=counted();
pub const CAPTURE:usize=captured(21);
#[unsafe(no_mangle)]pub extern "C" fn observed_count()->usize {COUNT}
#[unsafe(no_mangle)]pub extern "C" fn observed_capture()->usize {CAPTURE}
#[unsafe(no_mangle)]pub extern "C" fn observed_runtime(value:usize)->usize {value+counted()}
