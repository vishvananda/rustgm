mod definitions{pub mod u8{}}use definitions::u8;trait Build{fn build()->u8;}impl Build for u8{fn build()->u8{7}}fn main(){let _=u8::build();}
