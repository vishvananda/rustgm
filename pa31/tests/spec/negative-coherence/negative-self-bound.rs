#![feature(negative_impls)]
trait A where Self:Copy{}struct N;impl !A for N{}fn main(){}
