#![feature(no_core, pointer_metadata_fixture)]
#![no_core]
extern crate base;

struct Local;

impl base::Pointee for Local {
    type Metadata = ();
}
