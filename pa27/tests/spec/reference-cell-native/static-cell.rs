static S:u32=37;static REF:&u32=&S;const R:&&u32={let p=&raw const REF;unsafe{&*p}};fn main(){check(**R==37);}
