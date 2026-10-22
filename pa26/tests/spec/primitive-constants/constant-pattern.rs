fn check(b:bool){if !b{let z=0;let _=1/z;}}const M:u32=u32::MAX;fn f(x:u32)->bool{match x{M=>true,_=>false}}fn main(){check(f(4294967295u32));check(!f(0));}
