fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn pick<'a>(a:&'a u64,b:&'a u64,c:bool)->&'a u64{if c{a}else{b}}fn main(){let a=7;let b=9;check(*pick(&a,&b,true)==7 && *pick(&a,&b,false)==9);}
