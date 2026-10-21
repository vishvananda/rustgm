union U{a:u32,b:u8}union C{a:u32,b:char}union B{a:u8,b:bool}
const X:u32={let mut u=U{b:1};let p=&raw mut u.a;unsafe{*p=37;*p}};
const Y:char={let mut u=C{b:'a'};let p=&raw mut u.a;let q=&raw const u.b;unsafe{*p=0x3bb;*q}};
const Z:bool={let mut u=B{b:false};let p=&raw mut u.a;let q=&raw const u.b;unsafe{*p=1;*q}};
fn main(){check(X==37);check(Y=='λ');check(Z);}
