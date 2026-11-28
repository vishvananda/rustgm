fn temp()->u8{7}fn take(x:&u8)->u8{*x}fn main(){let n=take({&temp()});let _=1u8/((n==7)as u8);}
