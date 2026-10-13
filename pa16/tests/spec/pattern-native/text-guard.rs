fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&str,b:bool)->u8{match s{"héllo" if b=>7,"héllo"=>9,""=>1,_=>0}}fn main(){check(f("héllo",true)==7 && f("héllo",false)==9 && f("",false)==1 && f("x",true)==0);}
