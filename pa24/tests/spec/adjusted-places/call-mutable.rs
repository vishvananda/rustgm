fn main(){let mut x=7u8;let mut g=||{x+=1;};let r=&mut g;let mut f=||r();f();}
