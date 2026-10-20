trait A{}impl A for u8{}fn inner()->impl A{7u8}fn outer()->impl A+Send{inner()}
fn main(){}
