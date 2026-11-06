#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared};
type AliasMut<'a>=&'a mut u32;
type AliasRef<'a>=&'a u32;
struct Source<'a>{value:AliasMut<'a>}
impl<'a>Reborrow for Source<'a>{}
#[derive(Clone,Copy)]struct Target<'a>{value:AliasRef<'a>}
impl<'a>CoerceShared<Target<'a>>for Source<'a>{}
