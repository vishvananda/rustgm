#![feature(no_core)]
#![no_core]
extern crate base;

const ASSOCIATED: (u8, u8, u8) = char::INLINE_VERSION;
const FORWARDED: (u8, u8, u8) = base::VERSION;
const MIXED: (u8, bool, u16) = base::MIXED;
const FLAG: u8 = if MIXED.1 { 9 } else { 19 };
const MAJOR: u8 = ASSOCIATED.0;
const MINOR: u8 = ASSOCIATED.1;
const PATCH: u8 = ASSOCIATED.2;
const FORWARDED_PATCH: u8 = FORWARDED.2;
const WORD: u16 = MIXED.2;

pub fn major() -> u8 { MAJOR }
pub fn minor() -> u8 { MINOR }
pub fn patch() -> u8 { PATCH }
pub fn forwarded() -> u8 { FORWARDED_PATCH }
pub fn flag() -> u8 { FLAG }
pub fn word() -> u16 { WORD }
