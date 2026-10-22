fn cast<'a>(p:&'a &'a [u8;2])->&'a &'a [u8]{p as &'a &'a [u8]}fn main(){}
