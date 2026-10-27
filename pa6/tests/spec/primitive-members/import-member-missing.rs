mod u8{}trait Build{fn build()->u8;}impl Build for u8{fn build()->u8{7}}use u8::build;fn main(){let _=build();}
