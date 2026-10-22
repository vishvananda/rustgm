#[repr(C)]pub struct Mixed{pub n:u64,pub x:f64}
#[repr(C)]pub struct Nested{pub cells:[[u16;2];2],pub tag:u64}
#[repr(C)]pub struct Large{pub cells:[u64;5]}
#[repr(C)]pub struct Private{pub visible:u64,hidden:u64}
#[unsafe(no_mangle)]pub extern "C" fn metadata_mixed(value:Mixed)->Mixed{Mixed{n:value.n+3,x:value.x+2.5}}
#[unsafe(no_mangle)]pub extern "C" fn metadata_nested(mut value:Nested)->Nested{value.cells[1][1]+=1;value.tag+=2;value}
#[unsafe(no_mangle)]pub extern "C" fn metadata_large(mut value:Large)->Large{value.cells[4]+=7;value}
pub extern "C" fn private_value()->Private{Private{visible:7,hidden:9}}
