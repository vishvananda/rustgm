struct S{v:u32}const N:u32={let mut a=S{v:0};let mut b=S{v:0};let mut p=&raw mut a;let mut i=0;while i<2{if i==1{p=&raw mut b}let q=unsafe{&raw mut (*p).v};unsafe{*q=i+10}i+=1;}a.v+b.v};
fn main(){check(N==21)}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
