trait Pair<'a,'b>{}impl<'a>Pair<'a,'a> for u8{}fn f()->impl for<'a,'b>Pair<'a,'b>{7u8}fn main(){}
