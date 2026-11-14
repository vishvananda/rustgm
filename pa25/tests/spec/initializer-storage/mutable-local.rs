const fn word(x:u8)->u8{let mut y=x;let p=&mut y;*p+=2;y}static DATA:&u8=&word(7);fn main(){if *DATA!=9{let x=0u8;let _=1/x;}}
