fn main(){let r=&mut 7u8;let f=move||{let old=r;r=old;};f();}
