#[repr(C,packed)]struct P{prefix:u8,p:*const u32}const X:u32={let x=37u32;let outer=P{prefix:0,p:&raw const x};let p=&raw const outer;unsafe{*((*p).p)}};fn main(){let _=X;}
