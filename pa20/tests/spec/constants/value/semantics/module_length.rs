fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{struct P;impl P{const N:usize=2;}const _:[();2]=[();P::N];pub fn n()->usize{P::N}}fn main(){check(m::n()==2);}
