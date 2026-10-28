fn read(x:&u8)->u8{*x}
fn f(r:&mut u8){let s:&u8=r;*r=2;read(s);}
