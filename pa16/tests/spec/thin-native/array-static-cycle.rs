fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct N{all:&'static [N;2],v:u8}static A:[N;2]=[N{all:&A,v:3},N{all:&A,v:4}];const R:&u8=&A[1].v;fn main(){check(A[0].all[1].all[0].v==3 && *R==4);}
