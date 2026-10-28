fn read(x:&u8)->u8{*x}
fn f(r:&u8,b:bool){read(match b{true=>r,false=>r});read(r);}
