#[repr(C,packed)]struct Packed{prefix:u8,value:u32}fn main(){let packed=Packed{prefix:0,value:7};let Packed{value,..}=packed;let _=value;}
