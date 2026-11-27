#![feature(async_drop)]
#![allow(incomplete_features,dead_code,unused_variables,unused_imports)]
use core::pin::Pin;use core::future::AsyncDrop as Actual;
struct Token<T>(T);impl<T> Drop for Token<T>{fn drop(&mut self){}}
impl<T> Actual for Token<T>{async fn drop(self:Pin<&mut Self>){}}