enum E{A(u32),Empty}const N:u32={let mut e=E::A(37);let mut i=0;while i<8{match &mut e{E::A(x)=>{let v=*x;let p=&raw mut *x;unsafe{*p=v+1}},_=>{}}i+=1;}match e{E::A(x)=>x,_=>0}};
fn main(){check(N==45)}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
