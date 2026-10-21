fn main(){let ref mut r=7u8;let f=move||{let old=r;r=old;};f();}
