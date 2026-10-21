#[repr(C,packed)]struct Packed{prefix:u8,value:u32}fn main(){let packed=Packed{prefix:0,value:7};match packed{Packed{value:ref borrow,..}=>{let _=borrow;}}}
