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

fn require<T:PointeeSized+Unsize<U>,U:PointeeSized>(){}
trait Flag{type V0;type V1;type V2;type V3;type V4;type V5;type V6;type V7;type V8;type V9;type V10;type V11;type V12;type V13;type V14;type V15;type V16;type V17;type V18;type V19;type V20;type V21;type V22;type V23;type V24;type V25;type V26;type V27;type V28;type V29;type V30;type V31;type V32;type V33;type V34;type V35;type V36;type V37;type V38;type V39;type V40;type V41;type V42;type V43;type V44;type V45;type V46;type V47;type V48;type V49;type V50;type V51;type V52;type V53;type V54;type V55;type V56;type V57;type V58;type V59;type V60;type V61;type V62;type V63;type V64;type V65;type V66;type V67;type V68;type V69;type V70;type V71;type V72;type V73;type V74;type V75;type V76;type V77;type V78;type V79;type V80;type V81;type V82;type V83;type V84;type V85;type V86;type V87;type V88;type V89;type V90;type V91;type V92;type V93;type V94;type V95;type V96;type V97;type V98;type V99;type V100;type V101;type V102;type V103;type V104;type V105;type V106;type V107;type V108;type V109;type V110;type V111;type V112;type V113;type V114;type V115;type V116;type V117;type V118;type V119;type V120;type V121;type V122;type V123;type V124;type V125;type V126;type V127;}struct A;impl Flag for A{type V0=u8;type V1=u8;type V2=u8;type V3=u8;type V4=u8;type V5=u8;type V6=u8;type V7=u8;type V8=u8;type V9=u8;type V10=u8;type V11=u8;type V12=u8;type V13=u8;type V14=u8;type V15=u8;type V16=u8;type V17=u8;type V18=u8;type V19=u8;type V20=u8;type V21=u8;type V22=u8;type V23=u8;type V24=u8;type V25=u8;type V26=u8;type V27=u8;type V28=u8;type V29=u8;type V30=u8;type V31=u8;type V32=u8;type V33=u8;type V34=u8;type V35=u8;type V36=u8;type V37=u8;type V38=u8;type V39=u8;type V40=u8;type V41=u8;type V42=u8;type V43=u8;type V44=u8;type V45=u8;type V46=u8;type V47=u8;type V48=u8;type V49=u8;type V50=u8;type V51=u8;type V52=u8;type V53=u8;type V54=u8;type V55=u8;type V56=u8;type V57=u8;type V58=u8;type V59=u8;type V60=u8;type V61=u8;type V62=u8;type V63=u8;type V64=u8;type V65=u8;type V66=u8;type V67=u8;type V68=u8;type V69=u8;type V70=u8;type V71=u8;type V72=u8;type V73=u8;type V74=u8;type V75=u8;type V76=u8;type V77=u8;type V78=u8;type V79=u8;type V80=u8;type V81=u8;type V82=u8;type V83=u8;type V84=u8;type V85=u8;type V86=u8;type V87=u8;type V88=u8;type V89=u8;type V90=u8;type V91=u8;type V92=u8;type V93=u8;type V94=u8;type V95=u8;type V96=u8;type V97=u8;type V98=u8;type V99=u8;type V100=u8;type V101=u8;type V102=u8;type V103=u8;type V104=u8;type V105=u8;type V106=u8;type V107=u8;type V108=u8;type V109=u8;type V110=u8;type V111=u8;type V112=u8;type V113=u8;type V114=u8;type V115=u8;type V116=u8;type V117=u8;type V118=u8;type V119=u8;type V120=u8;type V121=u8;type V122=u8;type V123=u8;type V124=u8;type V125=u8;type V126=u8;type V127=u8;}
type Target=dyn Flag<V0=u8,V1=u8,V2=u8,V3=u8,V4=u8,V5=u8,V6=u8,V7=u8,V8=u8,V9=u8,V10=u8,V11=u8,V12=u8,V13=u8,V14=u8,V15=u8,V16=u8,V17=u8,V18=u8,V19=u8,V20=u8,V21=u8,V22=u8,V23=u8,V24=u8,V25=u8,V26=u8,V27=u8,V28=u8,V29=u8,V30=u8,V31=u8,V32=u8,V33=u8,V34=u8,V35=u8,V36=u8,V37=u8,V38=u8,V39=u8,V40=u8,V41=u8,V42=u8,V43=u8,V44=u8,V45=u8,V46=u8,V47=u8,V48=u8,V49=u8,V50=u8,V51=u8,V52=u8,V53=u8,V54=u8,V55=u8,V56=u8,V57=u8,V58=u8,V59=u8,V60=u8,V61=u8,V62=u8,V63=u8,V64=u8,V65=u8,V66=u8,V67=u8,V68=u8,V69=u8,V70=u8,V71=u8,V72=u8,V73=u8,V74=u8,V75=u8,V76=u8,V77=u8,V78=u8,V79=u8,V80=u8,V81=u8,V82=u8,V83=u8,V84=u8,V85=u8,V86=u8,V87=u8,V88=u8,V89=u8,V90=u8,V91=u8,V92=u8,V93=u8,V94=u8,V95=u8,V96=u8,V97=u8,V98=u8,V99=u8,V100=u8,V101=u8,V102=u8,V103=u8,V104=u8,V105=u8,V106=u8,V107=u8,V108=u8,V109=u8,V110=u8,V111=u8,V112=u8,V113=u8,V114=u8,V115=u8,V116=u8,V117=u8,V118=u8,V119=u8,V120=u8,V121=u8,V122=u8,V123=u8,V124=u8,V125=u8,V126=u8,V127=u8>;
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{require::<A,Target>();7}
