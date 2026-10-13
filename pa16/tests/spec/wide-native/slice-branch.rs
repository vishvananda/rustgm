fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'a>(a:&'a [u8],b:&'a [u8],c:bool)->&'a [u8]{if c{a}else{b}}fn main(){let a=[1];let b=[2,3];check(f(&a,&b,false)[1]==3 && f(&a,&b,true)[0]==1);}
