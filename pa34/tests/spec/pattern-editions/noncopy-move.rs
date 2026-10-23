fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S{value:u8}
fn main(){let values=[S{value:7}];let [mut x]=&values;check(x.value==7);x=S{value:9};check(x.value==9);}
