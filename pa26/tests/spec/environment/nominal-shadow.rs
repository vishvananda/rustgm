struct Option(u8);fn take(_:core::option::Option<&'static str>){}fn main(){let _=Option(7);take(option_env!("KEY"));}
