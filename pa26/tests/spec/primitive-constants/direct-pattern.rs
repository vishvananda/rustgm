fn check(b:bool){if !b{let z=0;let _=1/z;}}fn pick(x:u8)->bool{match x{u8::MAX=>true,_=>false}}fn main(){check(pick(255)&&!pick(254));}
