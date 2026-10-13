fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f()->u64{42}fn main(){let r=&f();if *r==42{check(*r==42)}else{check(false)}}
