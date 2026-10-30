#![feature(rustc_attrs)]
trait Base{}trait A:Base{}struct S;impl A for S{}
