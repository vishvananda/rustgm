trait Get<'a>{type Item;}impl<'a>Get<'a> for u8{type Item=&'a u8;}fn f()->impl for<'a>Get<'a,Item=&'static u8>{7u8}fn main(){}
