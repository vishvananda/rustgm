#![feature(async_drop)]
#![allow(incomplete_features,dead_code,unused_variables,unused_imports)]
use core::pin::Pin;use core::future::AsyncDrop as Actual;
struct Token;impl Actual for Token{async fn drop(self:Pin<&mut Self>){}}
