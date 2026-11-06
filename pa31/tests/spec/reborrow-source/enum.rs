#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared};
enum E<'a>{A(&'a mut u8)}impl<'a>Reborrow for E<'a>{}
