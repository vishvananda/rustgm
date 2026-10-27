#![feature(no_core,lang_items,rustc_attrs,staged_api)]#![no_core]#![rustc_coherence_is_core]#![stable(feature="char_test",since="1.0.0")]#[lang="pointee_sized"]pub trait PointeeSized{}#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}#[lang="sized"]pub trait Sized:MetaSized{}
const VALUE:char=char::MAX;
