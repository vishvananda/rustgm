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
/// Helper function for `drop_in_place`. The compiler replaces this by the actual drop glue.
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}


// Authored pure-data companion: no arithmetic runtime guard or replacement panic body.
const fn shared(value:usize)->usize {let call=const ||value;call()}
const fn mutable()->usize {let mut value=23usize;{let mut call=const ||{value=47usize;};call();}value}
struct Owned(usize);
const fn once(value:Owned)->usize {let call=const move ||value;let output=call();let _=call();output.0}
const fn make_shared(value:usize)->impl Fn()->usize {const move ||value}
const fn make_mut(mut value:usize)->impl FnMut(usize)->usize {const move |next:usize|{let old=value;value=next;old}}
pub const SHARED:usize=shared(23);
pub const MUTABLE:usize=mutable();
pub const ONCE:usize=once(Owned(23));
#[unsafe(no_mangle)]pub extern "C" fn pure_shared_constant()->usize {SHARED}
#[unsafe(no_mangle)]pub extern "C" fn pure_mutable_constant()->usize {MUTABLE}
#[unsafe(no_mangle)]pub extern "C" fn pure_once_constant()->usize {ONCE}
#[unsafe(no_mangle)]pub extern "C" fn pure_runtime_shared(value:usize)->usize {make_shared(value)()}
#[unsafe(no_mangle)]pub extern "C" fn pure_runtime_mutable(value:usize,next:usize)->usize {let mut call=make_mut(value);let _=call(next);call(59)}
