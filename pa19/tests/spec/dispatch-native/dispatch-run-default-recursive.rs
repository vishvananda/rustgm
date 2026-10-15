fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:Sized{fn f(self,n:u8)->u8{if n==0{7}else{A::f(self,n-1)}}}impl A for u8{}fn main(){let x=A::f(3u8,3);check(x==7);}