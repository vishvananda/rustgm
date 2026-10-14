struct S;impl Drop for S{#[cfg(any())]fn extra(){}#[cfg(all())]fn drop(&mut self){}}fn main(){}
