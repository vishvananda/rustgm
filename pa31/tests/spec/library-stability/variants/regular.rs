#![feature(no_core,lang_items,rustc_attrs,staged_api,stmt_expr_attributes)]
#![no_core]
#![unstable(feature="variant_fixture",issue="none")]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[stable(feature="enum_api",since="1.0.0")]
pub enum E{
#[stable(feature="variant_api",since="1.0.0")]A,
#[unstable(feature="variant_next",issue="none")]B,
}
pub fn make()->E{E::A}
