fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W(*mut u32);impl core::ops::Index<bool> for W{type Output=*mut u32;fn index(&self,_i:bool)->&Self::Output{&self.0}}fn main(){let mut n=1;let w=W(&raw mut n);unsafe{*w[false]=9;}check(n==9);}
