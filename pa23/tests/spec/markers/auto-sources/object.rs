trait A{}impl A for u8{}fn f(){let _: &(dyn A+Send+Sync)=&7u8;}
fn main(){}
