fn f(v:u32)->u32{v+1}union U{o:Option<fn(u32)->u32>,p:fn(u32)->u32}const N:Option<fn(u32)->u32>=unsafe{(U{p:f}).o};fn main(){check(match N{Some(f)=>f(36)==37,None=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
