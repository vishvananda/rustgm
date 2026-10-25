#![feature(unboxed_closures)]
#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,auto_traits)]
#![no_core]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a mut U> for &'b mut T{}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<*mut U> for *mut T{}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
pub mod intrinsics{use crate::Sized;#[rustc_intrinsic]pub const fn needs_drop<T:?Sized>()->bool;}
pub const fn needs_drop<T: ?Sized>() -> bool {
    const { intrinsics::needs_drop::<T>() }
}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang = "drop_glue"]
pub(crate) const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

impl Clone for u8{fn clone(&self)->Self{*self}}
impl Copy for u8{}
const fn read(x:&u8)->u8{*x}
type Read0=[u8;read(&7u8) as usize];
type Read1=[u8;read(&7u8) as usize];
type Read2=[u8;read(&7u8) as usize];
type Read3=[u8;read(&7u8) as usize];
type Read4=[u8;read(&7u8) as usize];
type Read5=[u8;read(&7u8) as usize];
type Read6=[u8;read(&7u8) as usize];
type Read7=[u8;read(&7u8) as usize];
type Read8=[u8;read(&7u8) as usize];
type Read9=[u8;read(&7u8) as usize];
type Read10=[u8;read(&7u8) as usize];
type Read11=[u8;read(&7u8) as usize];
type Read12=[u8;read(&7u8) as usize];
type Read13=[u8;read(&7u8) as usize];
type Read14=[u8;read(&7u8) as usize];
type Read15=[u8;read(&7u8) as usize];
type Read16=[u8;read(&7u8) as usize];
type Read17=[u8;read(&7u8) as usize];
type Read18=[u8;read(&7u8) as usize];
type Read19=[u8;read(&7u8) as usize];
type Read20=[u8;read(&7u8) as usize];
type Read21=[u8;read(&7u8) as usize];
type Read22=[u8;read(&7u8) as usize];
type Read23=[u8;read(&7u8) as usize];
type Read24=[u8;read(&7u8) as usize];
type Read25=[u8;read(&7u8) as usize];
type Read26=[u8;read(&7u8) as usize];
type Read27=[u8;read(&7u8) as usize];
type Read28=[u8;read(&7u8) as usize];
type Read29=[u8;read(&7u8) as usize];
type Read30=[u8;read(&7u8) as usize];
type Read31=[u8;read(&7u8) as usize];
type Read32=[u8;read(&7u8) as usize];
type Read33=[u8;read(&7u8) as usize];
type Read34=[u8;read(&7u8) as usize];
type Read35=[u8;read(&7u8) as usize];
type Read36=[u8;read(&7u8) as usize];
type Read37=[u8;read(&7u8) as usize];
type Read38=[u8;read(&7u8) as usize];
type Read39=[u8;read(&7u8) as usize];
type Read40=[u8;read(&7u8) as usize];
type Read41=[u8;read(&7u8) as usize];
type Read42=[u8;read(&7u8) as usize];
type Read43=[u8;read(&7u8) as usize];
type Read44=[u8;read(&7u8) as usize];
type Read45=[u8;read(&7u8) as usize];
type Read46=[u8;read(&7u8) as usize];
type Read47=[u8;read(&7u8) as usize];
type Read48=[u8;read(&7u8) as usize];
type Read49=[u8;read(&7u8) as usize];
type Read50=[u8;read(&7u8) as usize];
type Read51=[u8;read(&7u8) as usize];
type Read52=[u8;read(&7u8) as usize];
type Read53=[u8;read(&7u8) as usize];
type Read54=[u8;read(&7u8) as usize];
type Read55=[u8;read(&7u8) as usize];
type Read56=[u8;read(&7u8) as usize];
type Read57=[u8;read(&7u8) as usize];
type Read58=[u8;read(&7u8) as usize];
type Read59=[u8;read(&7u8) as usize];
type Read60=[u8;read(&7u8) as usize];
type Read61=[u8;read(&7u8) as usize];
type Read62=[u8;read(&7u8) as usize];
type Read63=[u8;read(&7u8) as usize];
type Read64=[u8;read(&7u8) as usize];
type Read65=[u8;read(&7u8) as usize];
type Read66=[u8;read(&7u8) as usize];
type Read67=[u8;read(&7u8) as usize];
type Read68=[u8;read(&7u8) as usize];
type Read69=[u8;read(&7u8) as usize];
type Read70=[u8;read(&7u8) as usize];
type Read71=[u8;read(&7u8) as usize];
type Read72=[u8;read(&7u8) as usize];
type Read73=[u8;read(&7u8) as usize];
type Read74=[u8;read(&7u8) as usize];
type Read75=[u8;read(&7u8) as usize];
type Read76=[u8;read(&7u8) as usize];
type Read77=[u8;read(&7u8) as usize];
type Read78=[u8;read(&7u8) as usize];
type Read79=[u8;read(&7u8) as usize];
type Read80=[u8;read(&7u8) as usize];
type Read81=[u8;read(&7u8) as usize];
type Read82=[u8;read(&7u8) as usize];
type Read83=[u8;read(&7u8) as usize];
type Read84=[u8;read(&7u8) as usize];
type Read85=[u8;read(&7u8) as usize];
type Read86=[u8;read(&7u8) as usize];
type Read87=[u8;read(&7u8) as usize];
type Read88=[u8;read(&7u8) as usize];
type Read89=[u8;read(&7u8) as usize];
type Read90=[u8;read(&7u8) as usize];
type Read91=[u8;read(&7u8) as usize];
type Read92=[u8;read(&7u8) as usize];
type Read93=[u8;read(&7u8) as usize];
type Read94=[u8;read(&7u8) as usize];
type Read95=[u8;read(&7u8) as usize];
type Read96=[u8;read(&7u8) as usize];
type Read97=[u8;read(&7u8) as usize];
type Read98=[u8;read(&7u8) as usize];
type Read99=[u8;read(&7u8) as usize];
type Read100=[u8;read(&7u8) as usize];
type Read101=[u8;read(&7u8) as usize];
type Read102=[u8;read(&7u8) as usize];
type Read103=[u8;read(&7u8) as usize];
type Read104=[u8;read(&7u8) as usize];
type Read105=[u8;read(&7u8) as usize];
type Read106=[u8;read(&7u8) as usize];
type Read107=[u8;read(&7u8) as usize];
type Read108=[u8;read(&7u8) as usize];
type Read109=[u8;read(&7u8) as usize];
type Read110=[u8;read(&7u8) as usize];
type Read111=[u8;read(&7u8) as usize];
type Read112=[u8;read(&7u8) as usize];
type Read113=[u8;read(&7u8) as usize];
type Read114=[u8;read(&7u8) as usize];
type Read115=[u8;read(&7u8) as usize];
type Read116=[u8;read(&7u8) as usize];
type Read117=[u8;read(&7u8) as usize];
type Read118=[u8;read(&7u8) as usize];
type Read119=[u8;read(&7u8) as usize];
type Read120=[u8;read(&7u8) as usize];
type Read121=[u8;read(&7u8) as usize];
type Read122=[u8;read(&7u8) as usize];
type Read123=[u8;read(&7u8) as usize];
type Read124=[u8;read(&7u8) as usize];
type Read125=[u8;read(&7u8) as usize];
type Read126=[u8;read(&7u8) as usize];
type Read127=[u8;read(&7u8) as usize];
type Early=Read0;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{let _:Early=[0u8;7];7}
#[unsafe(no_mangle)]pub extern "C" fn runtime()->u8{read(&7u8)}
