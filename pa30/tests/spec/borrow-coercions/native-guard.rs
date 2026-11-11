#![allow(dead_code,unused_variables)]
use core::ops::{Deref,DerefMut};
struct Guard(u8);
impl Deref for Guard{type Target=u8;fn deref(&self)->&u8{&self.0}}
impl DerefMut for Guard{fn deref_mut(&mut self)->&mut u8{let _=tick(3);&mut self.0}}
impl Drop for Guard{fn drop(&mut self){observe_drop(self.0);let _=tick(5);}}
unsafe extern "C"{safe fn tick(code:u8)->u8;safe fn observe_drop(n:u8);}
fn make(n:u8)->Guard{let _=tick(1);Guard(n)}fn argument()->u8{tick(4)}
fn replace(x:&mut u8,n:u8)->u8{let old=*x;*x=n;old}
#[unsafe(no_mangle)]pub extern "C" fn observed_guard(n:u8)->u8{replace(&mut make(n),argument())}
