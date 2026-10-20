trait Read<'a>{}impl Read<'static> for u8{}fn f()->impl for<'a>Read<'a>{7u8}fn main(){}
