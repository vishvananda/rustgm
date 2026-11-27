#![allow(dead_code,unused_unsafe,internal_features)]
trait T{#[target_feature(enable="sse2")]unsafe fn f(){}}
