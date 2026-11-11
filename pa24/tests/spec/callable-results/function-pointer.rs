fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R>(f:fn()->R)->R{f()}fn project<T>(f:fn()->(T,))->T{drive(f).0}fn value()->(u8,){(23,)}
fn main(){check(project(value)==23);}
