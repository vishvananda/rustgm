fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(<i128 as core::ops::Div>::div(-36893488147419103232,2)==-18446744073709551616);check(<i128 as core::ops::Rem>::rem(-18446744073709551623,18446744073709551616)==-7);}
