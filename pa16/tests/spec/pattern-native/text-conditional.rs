fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&str)->u8{if let "a"=s{return 1;}let "b"=s else{return 0;};2}fn main(){check(f("a")==1 && f("b")==2 && f("c")==0);}
