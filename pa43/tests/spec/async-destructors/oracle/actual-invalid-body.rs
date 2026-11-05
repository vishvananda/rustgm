#![feature(async_drop)]
#![allow(incomplete_features,dead_code,unused_variables,unused_imports)]
use core::pin::Pin;use core::future::AsyncDrop as Actual;
struct Token;
impl Drop for Token{fn drop(&mut self){}}
impl Actual for Token{async fn drop(self:Pin<&mut Self>){let _:u8=true;}}
