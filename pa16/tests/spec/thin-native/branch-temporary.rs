fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(c:bool){let r=&(if c{7u64}else{9});check(*r==if c{7}else{9});}fn main(){f(true);f(false);}
