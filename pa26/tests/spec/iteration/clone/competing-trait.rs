struct V(u8);trait Other{fn clone(&self)->u8;}impl<'a,T> Other for core::slice::Iter<'a,T>{fn clone(&self)->u8{9}}fn main(){let a=[V(1)];let _=a.iter().clone();}
