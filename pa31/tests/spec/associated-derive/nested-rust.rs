trait Family{type Item;}
#[derive(Clone)]struct Nested<I>where I:Family<Item=Option<Result<u16,u8>>>{value:I}
