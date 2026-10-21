fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct P{ptr:*mut u32}struct W(P);impl core::ops::Index<bool> for W{type Output=P;fn index(&self,_i:bool)->&P{&self.0}}fn main(){let mut n=1;let w=W(P{ptr:&raw mut n});unsafe{*w[false].ptr=9;}check(n==9);}
