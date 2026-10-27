mod scope{pub mod u8{}}trait Build{fn build()->u8;}impl Build for u8{fn build()->u8{7}}fn main(){let _=scope::u8::build();}
