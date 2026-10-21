#[repr(C,packed)]struct Packed{prefix:u8,value:u32}fn main(){let mut packed=Packed{prefix:0,value:7};packed.value+=1;let value=packed.value;let _=value;}
