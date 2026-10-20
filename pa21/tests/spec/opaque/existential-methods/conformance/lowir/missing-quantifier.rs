trait V<'a>{}impl V<'static> for u8{}trait A{fn f()->impl for<'a>V<'a>;}impl A for u16{fn f()->u8{7}}fn main(){}
