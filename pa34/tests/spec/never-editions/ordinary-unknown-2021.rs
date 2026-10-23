enum Maybe<T>{None,Some(T)}
fn probe()->u8{let _x=Maybe::None;7}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(probe()==7);}
