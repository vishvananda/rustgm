fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod a{mod b{pub(super) struct P;impl P{pub(super) const N:usize=2;}}const _:[();2]=[();b::P::N];pub fn n()->usize{b::P::N}}fn main(){check(a::n()==2);}
