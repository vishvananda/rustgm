fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[repr(C)]#[derive(Clone,Copy)]struct I{a:u64,b:u8} #[repr(C,packed)]struct S{tag:u8,i:I} fn id(x:S)->S{x} fn main(){let mut s=id(S{tag:2,i:I{a:444,b:8}});let i=s.i;s.i=I{a:555,b:9};check(i.a==444 && s.i.a==555 && s.i.b==9);}
