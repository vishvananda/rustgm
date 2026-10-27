#![feature(no_core,lang_items,macro_metavar_expr_concat)]
#![no_core]
#![allow(internal_features)]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
macro_rules!m{($n:ident)=>{{let ${concat($n,_x)}=7u8;let _=${concat($n,_x)};}}}fn use_it(){m!(a);}
