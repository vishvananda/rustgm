fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Code{fn code(self)->u32;}
enum Opt<T>{Some(T),None}
fn run<F:FnOnce()>(f:F){f()}
fn main(){let v=3u8;let mut s=Opt::None;run(||{s=Opt::Some(&v);});check(match s{Opt::Some(r)=>*r==3,Opt::None=>false});}
