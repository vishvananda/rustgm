#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl)]
#![no_core]
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

struct Leaf;impl Drop for Leaf{fn drop(&mut self){}}
struct Root(Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf,Leaf);
const Q0:bool=needs_drop::<Root>();type Check0=[u8;Q0 as usize];
const Q1:bool=needs_drop::<Root>();type Check1=[u8;Q1 as usize];
const Q2:bool=needs_drop::<Root>();type Check2=[u8;Q2 as usize];
const Q3:bool=needs_drop::<Root>();type Check3=[u8;Q3 as usize];
const Q4:bool=needs_drop::<Root>();type Check4=[u8;Q4 as usize];
const Q5:bool=needs_drop::<Root>();type Check5=[u8;Q5 as usize];
const Q6:bool=needs_drop::<Root>();type Check6=[u8;Q6 as usize];
const Q7:bool=needs_drop::<Root>();type Check7=[u8;Q7 as usize];
const Q8:bool=needs_drop::<Root>();type Check8=[u8;Q8 as usize];
const Q9:bool=needs_drop::<Root>();type Check9=[u8;Q9 as usize];
const Q10:bool=needs_drop::<Root>();type Check10=[u8;Q10 as usize];
const Q11:bool=needs_drop::<Root>();type Check11=[u8;Q11 as usize];
const Q12:bool=needs_drop::<Root>();type Check12=[u8;Q12 as usize];
const Q13:bool=needs_drop::<Root>();type Check13=[u8;Q13 as usize];
const Q14:bool=needs_drop::<Root>();type Check14=[u8;Q14 as usize];
const Q15:bool=needs_drop::<Root>();type Check15=[u8;Q15 as usize];
const Q16:bool=needs_drop::<Root>();type Check16=[u8;Q16 as usize];
const Q17:bool=needs_drop::<Root>();type Check17=[u8;Q17 as usize];
const Q18:bool=needs_drop::<Root>();type Check18=[u8;Q18 as usize];
const Q19:bool=needs_drop::<Root>();type Check19=[u8;Q19 as usize];
const Q20:bool=needs_drop::<Root>();type Check20=[u8;Q20 as usize];
const Q21:bool=needs_drop::<Root>();type Check21=[u8;Q21 as usize];
const Q22:bool=needs_drop::<Root>();type Check22=[u8;Q22 as usize];
const Q23:bool=needs_drop::<Root>();type Check23=[u8;Q23 as usize];
const Q24:bool=needs_drop::<Root>();type Check24=[u8;Q24 as usize];
const Q25:bool=needs_drop::<Root>();type Check25=[u8;Q25 as usize];
const Q26:bool=needs_drop::<Root>();type Check26=[u8;Q26 as usize];
const Q27:bool=needs_drop::<Root>();type Check27=[u8;Q27 as usize];
const Q28:bool=needs_drop::<Root>();type Check28=[u8;Q28 as usize];
const Q29:bool=needs_drop::<Root>();type Check29=[u8;Q29 as usize];
const Q30:bool=needs_drop::<Root>();type Check30=[u8;Q30 as usize];
const Q31:bool=needs_drop::<Root>();type Check31=[u8;Q31 as usize];
const Q32:bool=needs_drop::<Root>();type Check32=[u8;Q32 as usize];
const Q33:bool=needs_drop::<Root>();type Check33=[u8;Q33 as usize];
const Q34:bool=needs_drop::<Root>();type Check34=[u8;Q34 as usize];
const Q35:bool=needs_drop::<Root>();type Check35=[u8;Q35 as usize];
const Q36:bool=needs_drop::<Root>();type Check36=[u8;Q36 as usize];
const Q37:bool=needs_drop::<Root>();type Check37=[u8;Q37 as usize];
const Q38:bool=needs_drop::<Root>();type Check38=[u8;Q38 as usize];
const Q39:bool=needs_drop::<Root>();type Check39=[u8;Q39 as usize];
const Q40:bool=needs_drop::<Root>();type Check40=[u8;Q40 as usize];
const Q41:bool=needs_drop::<Root>();type Check41=[u8;Q41 as usize];
const Q42:bool=needs_drop::<Root>();type Check42=[u8;Q42 as usize];
const Q43:bool=needs_drop::<Root>();type Check43=[u8;Q43 as usize];
const Q44:bool=needs_drop::<Root>();type Check44=[u8;Q44 as usize];
const Q45:bool=needs_drop::<Root>();type Check45=[u8;Q45 as usize];
const Q46:bool=needs_drop::<Root>();type Check46=[u8;Q46 as usize];
const Q47:bool=needs_drop::<Root>();type Check47=[u8;Q47 as usize];
const Q48:bool=needs_drop::<Root>();type Check48=[u8;Q48 as usize];
const Q49:bool=needs_drop::<Root>();type Check49=[u8;Q49 as usize];
const Q50:bool=needs_drop::<Root>();type Check50=[u8;Q50 as usize];
const Q51:bool=needs_drop::<Root>();type Check51=[u8;Q51 as usize];
const Q52:bool=needs_drop::<Root>();type Check52=[u8;Q52 as usize];
const Q53:bool=needs_drop::<Root>();type Check53=[u8;Q53 as usize];
const Q54:bool=needs_drop::<Root>();type Check54=[u8;Q54 as usize];
const Q55:bool=needs_drop::<Root>();type Check55=[u8;Q55 as usize];
const Q56:bool=needs_drop::<Root>();type Check56=[u8;Q56 as usize];
const Q57:bool=needs_drop::<Root>();type Check57=[u8;Q57 as usize];
const Q58:bool=needs_drop::<Root>();type Check58=[u8;Q58 as usize];
const Q59:bool=needs_drop::<Root>();type Check59=[u8;Q59 as usize];
const Q60:bool=needs_drop::<Root>();type Check60=[u8;Q60 as usize];
const Q61:bool=needs_drop::<Root>();type Check61=[u8;Q61 as usize];
const Q62:bool=needs_drop::<Root>();type Check62=[u8;Q62 as usize];
const Q63:bool=needs_drop::<Root>();type Check63=[u8;Q63 as usize];
const Q64:bool=needs_drop::<Root>();type Check64=[u8;Q64 as usize];
const Q65:bool=needs_drop::<Root>();type Check65=[u8;Q65 as usize];
const Q66:bool=needs_drop::<Root>();type Check66=[u8;Q66 as usize];
const Q67:bool=needs_drop::<Root>();type Check67=[u8;Q67 as usize];
const Q68:bool=needs_drop::<Root>();type Check68=[u8;Q68 as usize];
const Q69:bool=needs_drop::<Root>();type Check69=[u8;Q69 as usize];
const Q70:bool=needs_drop::<Root>();type Check70=[u8;Q70 as usize];
const Q71:bool=needs_drop::<Root>();type Check71=[u8;Q71 as usize];
const Q72:bool=needs_drop::<Root>();type Check72=[u8;Q72 as usize];
const Q73:bool=needs_drop::<Root>();type Check73=[u8;Q73 as usize];
const Q74:bool=needs_drop::<Root>();type Check74=[u8;Q74 as usize];
const Q75:bool=needs_drop::<Root>();type Check75=[u8;Q75 as usize];
const Q76:bool=needs_drop::<Root>();type Check76=[u8;Q76 as usize];
const Q77:bool=needs_drop::<Root>();type Check77=[u8;Q77 as usize];
const Q78:bool=needs_drop::<Root>();type Check78=[u8;Q78 as usize];
const Q79:bool=needs_drop::<Root>();type Check79=[u8;Q79 as usize];
const Q80:bool=needs_drop::<Root>();type Check80=[u8;Q80 as usize];
const Q81:bool=needs_drop::<Root>();type Check81=[u8;Q81 as usize];
const Q82:bool=needs_drop::<Root>();type Check82=[u8;Q82 as usize];
const Q83:bool=needs_drop::<Root>();type Check83=[u8;Q83 as usize];
const Q84:bool=needs_drop::<Root>();type Check84=[u8;Q84 as usize];
const Q85:bool=needs_drop::<Root>();type Check85=[u8;Q85 as usize];
const Q86:bool=needs_drop::<Root>();type Check86=[u8;Q86 as usize];
const Q87:bool=needs_drop::<Root>();type Check87=[u8;Q87 as usize];
const Q88:bool=needs_drop::<Root>();type Check88=[u8;Q88 as usize];
const Q89:bool=needs_drop::<Root>();type Check89=[u8;Q89 as usize];
const Q90:bool=needs_drop::<Root>();type Check90=[u8;Q90 as usize];
const Q91:bool=needs_drop::<Root>();type Check91=[u8;Q91 as usize];
const Q92:bool=needs_drop::<Root>();type Check92=[u8;Q92 as usize];
const Q93:bool=needs_drop::<Root>();type Check93=[u8;Q93 as usize];
const Q94:bool=needs_drop::<Root>();type Check94=[u8;Q94 as usize];
const Q95:bool=needs_drop::<Root>();type Check95=[u8;Q95 as usize];
const Q96:bool=needs_drop::<Root>();type Check96=[u8;Q96 as usize];
const Q97:bool=needs_drop::<Root>();type Check97=[u8;Q97 as usize];
const Q98:bool=needs_drop::<Root>();type Check98=[u8;Q98 as usize];
const Q99:bool=needs_drop::<Root>();type Check99=[u8;Q99 as usize];
const Q100:bool=needs_drop::<Root>();type Check100=[u8;Q100 as usize];
const Q101:bool=needs_drop::<Root>();type Check101=[u8;Q101 as usize];
const Q102:bool=needs_drop::<Root>();type Check102=[u8;Q102 as usize];
const Q103:bool=needs_drop::<Root>();type Check103=[u8;Q103 as usize];
const Q104:bool=needs_drop::<Root>();type Check104=[u8;Q104 as usize];
const Q105:bool=needs_drop::<Root>();type Check105=[u8;Q105 as usize];
const Q106:bool=needs_drop::<Root>();type Check106=[u8;Q106 as usize];
const Q107:bool=needs_drop::<Root>();type Check107=[u8;Q107 as usize];
const Q108:bool=needs_drop::<Root>();type Check108=[u8;Q108 as usize];
const Q109:bool=needs_drop::<Root>();type Check109=[u8;Q109 as usize];
const Q110:bool=needs_drop::<Root>();type Check110=[u8;Q110 as usize];
const Q111:bool=needs_drop::<Root>();type Check111=[u8;Q111 as usize];
const Q112:bool=needs_drop::<Root>();type Check112=[u8;Q112 as usize];
const Q113:bool=needs_drop::<Root>();type Check113=[u8;Q113 as usize];
const Q114:bool=needs_drop::<Root>();type Check114=[u8;Q114 as usize];
const Q115:bool=needs_drop::<Root>();type Check115=[u8;Q115 as usize];
const Q116:bool=needs_drop::<Root>();type Check116=[u8;Q116 as usize];
const Q117:bool=needs_drop::<Root>();type Check117=[u8;Q117 as usize];
const Q118:bool=needs_drop::<Root>();type Check118=[u8;Q118 as usize];
const Q119:bool=needs_drop::<Root>();type Check119=[u8;Q119 as usize];
const Q120:bool=needs_drop::<Root>();type Check120=[u8;Q120 as usize];
const Q121:bool=needs_drop::<Root>();type Check121=[u8;Q121 as usize];
const Q122:bool=needs_drop::<Root>();type Check122=[u8;Q122 as usize];
const Q123:bool=needs_drop::<Root>();type Check123=[u8;Q123 as usize];
const Q124:bool=needs_drop::<Root>();type Check124=[u8;Q124 as usize];
const Q125:bool=needs_drop::<Root>();type Check125=[u8;Q125 as usize];
const Q126:bool=needs_drop::<Root>();type Check126=[u8;Q126 as usize];
const Q127:bool=needs_drop::<Root>();type Check127=[u8;Q127 as usize];
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{Q127 as u8}
