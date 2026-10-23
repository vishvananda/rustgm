#![feature(staged_api)]
#![stable(feature="fixture_root",since="1.0.0")]
#![feature(allow_internal_unstable)]
#[unstable(feature="fixture_private",issue="none")]
pub fn experimental()->u8{7}
#[stable(feature="fixture_root",since="1.0.0")]
#[macro_export]#[allow_internal_unstable(fixture_private)]macro_rules! allowed{()=>{$crate::experimental()}}
#[stable(feature="fixture_root",since="1.0.0")]
#[macro_export]macro_rules! plain{()=>{$crate::experimental()}}
#[stable(feature="fixture_root",since="1.0.0")]
#[macro_export]#[allow_internal_unstable(fixture_private)]macro_rules! pass{($e:expr)=>{$e}}
#[stable(feature="fixture_root",since="1.0.0")]
#[macro_export]macro_rules! nested{()=>{$crate::allowed!()}}
