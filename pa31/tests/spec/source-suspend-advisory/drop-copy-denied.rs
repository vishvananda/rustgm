#![feature(must_not_suspend)]
#[must_not_suspend]pub struct S(u32);impl Drop for S{fn drop(&mut self){}}impl Copy for S{}impl Clone for S{fn clone(&self)->Self{S(self.0)}}
