#![feature(cfg_sanitize)]
#[cfg(any(sanitize="cfi",sanitize="kcfi"))]const SELECT:bool=true;
#[cfg(not(any(sanitize="cfi",sanitize="kcfi")))]const SELECT:bool=false;
const CHECK:[u8;1]=[0;SELECT as usize];
