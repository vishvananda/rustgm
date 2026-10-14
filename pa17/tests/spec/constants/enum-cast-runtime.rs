enum E{A=3}impl Drop for E{fn drop(&mut self){}}fn f(e:E)->usize{e as usize}fn main(){let _=f(E::A);}
