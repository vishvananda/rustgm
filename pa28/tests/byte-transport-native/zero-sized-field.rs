union U{bits:u8,value:bool} struct S{empty:[();1_000_000_000_000],value:bool} const X:S={let source=U{bits:2};let mut target=S{empty:[();1_000_000_000_000],value:false};unsafe{core::ptr::copy(&raw const source.value,&raw mut target.value,1);}target.value=true;target};
fn main(){check(X.value);}
