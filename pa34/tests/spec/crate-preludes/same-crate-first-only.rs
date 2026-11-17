#![feature(prelude_import)]
#![no_std]
#![no_main]
pub mod earlier { pub struct EarlierOnly; pub struct Same; }
pub mod later { pub struct LaterOnly; pub struct Same; }
#[prelude_import]
use crate::earlier::*;
#[prelude_import]
use crate::later::*;
pub fn selected()->crate::later::Same { Same }
pub fn available()->crate::later::LaterOnly { LaterOnly }
pub fn absent()->crate::earlier::EarlierOnly { EarlierOnly }
