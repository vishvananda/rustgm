const fn make()->impl Copy{[7u8;128]}
const N:u32={let x=make();let p=&raw const x as *const [u8;128];let mut n=0u32;let mut i=0;while i<128{n+=unsafe{(*p)[127]} as u32;i+=1;}n};
fn main(){if N!=896{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
