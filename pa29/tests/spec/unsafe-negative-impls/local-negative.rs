#![feature(no_core,negative_impls)]
#![no_core]
extern crate base;
fn need<T: base::Error>() {}
struct Mine;
impl !base::Error for Mine {}
impl !base::Searcher for Mine {}
impl !base::DerefMutLike for Mine {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<base::Fault>(); 42 }
