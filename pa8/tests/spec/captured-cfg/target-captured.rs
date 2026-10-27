#![feature(cfg_target_has_atomic)]
macro_rules! atomic{($s:literal)=>{#[cfg(target_has_atomic_load_store=$s)]fn selected(){}};}atomic!("8");fn main(){selected();}
