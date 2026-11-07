// rustc injects `extern crate compiler_builtins` into every `no_std` crate
// that has no sysroot; this empty crate stands in for it in the rustc oracle.
#![feature(compiler_builtins)]
#![compiler_builtins]
#![no_std]
