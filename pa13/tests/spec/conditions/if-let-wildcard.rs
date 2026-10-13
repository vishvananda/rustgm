fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let x:u8;let y=if let _=x{7}else{9};let z=if let 0..=255=x{3}else{5};check(y==7&&z==3);}