pub static IMM:u32=7;
pub static mut MUT:u32=11;
#[unsafe(export_name="rustgm_storage_exact")]pub static EXACT:u64=13;
pub static REAL:f64=3.5;
pub static FLAG:bool=true;
pub static LETTER:char='x';
pub static WIDE:u128=340282366920938463463374607431768211453u128;
pub struct Record{pub n:u32}
pub static RECORD:Record=Record{n:17};
pub static REF:&u32=&IMM;
static PRIVATE:u32=19;
unsafe extern "C"{pub safe static SAFE:u32;pub static FOREIGN:u32;pub static mut EXTERNAL_MUT:u32;}
pub fn private_value()->u32{PRIVATE}
