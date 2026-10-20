trait Get{type Item;}impl Get for u8{type Item=u16;}fn f()->impl Get<Item=u8>{7u8}fn main(){}
