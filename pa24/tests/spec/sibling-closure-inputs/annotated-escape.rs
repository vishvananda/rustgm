fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Code{fn code(self)->u32;}
enum Opt<T>{Some(T),None}
fn main(){let v=3u8;let mut s=Opt::None;let mut f=|x:&u8|{s=Opt::Some(x);};f(&v);check(match s{Opt::Some(r)=>*r==3,Opt::None=>false});}
