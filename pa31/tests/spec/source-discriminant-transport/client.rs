#![feature(no_core, disc_fixture, discriminant_kind)]
#![no_core]
extern crate base;
use base::DiscriminantKind;

enum LocalTag { First, Second }

type External = <base::ExternalTag as DiscriminantKind>::Discriminant;
type Local = <LocalTag as DiscriminantKind>::Discriminant;
type Record = <base::ExternalRecord as DiscriminantKind>::Discriminant;

const EXTERNAL: External = 513u16;
const LOCAL: Local = 7isize;
const RECORD: Record = 29u8;

#[unsafe(no_mangle)]
pub extern "C" fn observe_external() -> u16 { EXTERNAL }
#[unsafe(no_mangle)]
pub extern "C" fn observe_local() -> isize { LOCAL }
#[unsafe(no_mangle)]
pub extern "C" fn observe_record() -> u8 { RECORD }
