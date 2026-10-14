trait Get{fn get(&self)->u8;}struct S;impl Get for S{fn get(&self)->u8{7}}fn main(){let _=S.get();}
