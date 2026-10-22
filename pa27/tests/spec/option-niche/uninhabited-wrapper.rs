enum Void{}#[repr(transparent)]struct W(&'static u32,Void);fn main(){let a:[Option<W>;2]=[None,None];let p=(&raw const a[0])as usize;let q=(&raw const a[1])as usize;check(q-p==8)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
