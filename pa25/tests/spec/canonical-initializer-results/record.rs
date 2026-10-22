fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(PartialEq,Eq)]struct R<'a>{r:&'a u8,n:u8}struct S<'a>(&'a u8);impl<'a>S<'a>{const C:R<'a>={let n=&7;R{r:n,n:9}};}fn main(){check(match (R{r:&7,n:9}){S::<'static>::C=>true,_=>false});check(!match (R{r:&8,n:9}){S::<'static>::C=>true,_=>false});}
