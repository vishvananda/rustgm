const A:u8=base::through(7u8);const B:u16=base::reserve::<u16,bool>(513);fn fail(x:u8)->u8{1/x}fn main(){if A!=7||B!=513{let _=fail(0);}}
