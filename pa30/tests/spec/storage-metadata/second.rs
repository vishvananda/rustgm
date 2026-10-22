extern crate first;
pub static SECOND:u32=first::IMM+5;
pub fn read()->u32{first::IMM+SECOND+first::RECORD.n}
pub fn increment()->u32{unsafe{first::MUT+=1;first::MUT}}
