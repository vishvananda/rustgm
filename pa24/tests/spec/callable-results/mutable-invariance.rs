fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn drive<R,F:FnOnce()->R>(f:F)->R{f()}
fn project<'a,'b:'a,F:FnOnce()->(&'a mut &'a u8,)>(f:F)->&'a mut &'b u8{drive(f).0}
fn main(){}
