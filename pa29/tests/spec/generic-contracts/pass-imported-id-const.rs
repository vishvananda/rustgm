const A:u8=base::id::<u8>(9);const B:bool=base::id(true);fn fail(x:u8)->u8{1/x}fn main(){if A!=9||!B{let _=fail(0);}}
