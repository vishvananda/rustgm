fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let p=(1u8,2u16,(3u32,4u64));let (a,..,(b,c))=p;check(a==1&&b==3&&c==4);}