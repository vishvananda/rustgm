trait V<'a>{}impl<'a>V<'a> for u8{}trait A{fn f()->impl for<'a>V<'a>;}impl A for u16{fn f()->impl for<'b>V<'b>{7u8}}fn main(){}
