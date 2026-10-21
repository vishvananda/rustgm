struct S(u8);fn main(){let x=S(7);let mut f=move||x.0+=2;f();}
