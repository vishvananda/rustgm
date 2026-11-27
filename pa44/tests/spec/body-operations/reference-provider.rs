#![feature(async_drop)]
#![no_std]
pub use core::future::{Future,async_drop_in_place as cleanup};
pub struct Packet<T> { pub value: T }
pub fn grade<T: Copy>(score: i32, low: T, high: T) -> T { match score { 0..=9 => low, _ => high } }
pub fn band<T: Copy>(score: i32, a: T, b: T, c: T) -> T {
    match score { 0..=9 => a, 10..=99 => b, _ => c }
}
pub fn exact<T: Copy>(code: u8, hit: T, miss: T) -> T { match code { 7 => hit, _ => miss } }
pub fn either<T: Copy>(code: u8, a: T, b: T, c: T) -> T { match code { 1 | 3 => a, 7 => b, _ => c } }
pub fn letter<T: Copy>(c: char, yes: T, no: T) -> T { match c { 'a'..='z' => yes, _ => no } }
pub fn flag<T: Copy>(on: bool, yes: T, no: T) -> T { match on { true => yes, false => no } }
