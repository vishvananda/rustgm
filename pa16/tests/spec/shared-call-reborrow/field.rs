fn read(x:&u8)->u8{*x}
struct Pair<'a>{r:&'a u8} fn f(p:Pair<'_>){read(p.r);read(p.r);}
