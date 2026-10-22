fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){let x=[1u8,7];let p=(&raw const x)as *const [u8];let _=unsafe{p.add(1)};}
