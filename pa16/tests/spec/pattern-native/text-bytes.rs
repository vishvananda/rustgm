fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&str)->u8{match s{""=>1,"a"=>2,"ab"=>3,"a\0b"=>4,"héllo"=>5,"🦀"=>6,"e\u{301}"=>7,_=>0}}fn main(){check(f("")==1 && f("a")==2 && f("ab")==3 && f("a\0b")==4 && f("héllo")==5 && f("🦀")==6 && f("e\u{301}")==7 && f("é")==0 && f("ac")==0 && f("abc")==0 && f("a\0c")==0);}
