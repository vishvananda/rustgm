fn tap(x:u8)->u8{print!("e{x};");x}
fn main(){let first=3u8;let last=9u8;let s=format!("{last} {first} {last:x} {0}",tap(1),last=tap(2));println!("{s}");let _=last;}
