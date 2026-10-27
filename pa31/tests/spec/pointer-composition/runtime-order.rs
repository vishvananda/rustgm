#![feature(ptr_metadata,extern_types)]
#![allow(dead_code)]
use core::{ptr,slice};
static DATA:[u8;3]=[7,8,9];
unsafe extern "C"{fn observed_data()->*const u8;fn observed_meta()->usize;fn observed_thin_data()->*const u8;fn observed_unit();}
#[unsafe(no_mangle)]pub extern "C" fn run_slice_len()->usize{ptr::metadata(ptr::slice_from_raw_parts(unsafe{observed_data()},unsafe{observed_meta()}))}
#[unsafe(no_mangle)]pub extern "C" fn run_thin()->u8{let p=ptr::from_raw_parts::<u8>(unsafe{observed_thin_data()},unsafe{observed_unit()});unsafe{*p}}
