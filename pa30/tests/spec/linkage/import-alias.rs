unsafe extern "C" {#[link_name="imported_i32"]safe fn local_name(x:i32)->i32;}
fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
fn main(){check(local_name(39)==42);}
