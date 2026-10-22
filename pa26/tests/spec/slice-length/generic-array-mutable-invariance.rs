fn bad<'a:'b,'b:'c,'c,const N:usize>(x:&'c mut[&'a u8;N])->&'c mut[&'b u8]{let _=x.len();x}fn main(){}
