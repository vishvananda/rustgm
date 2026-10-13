fn g<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'a>(c:bool){let p=if c{g::<'a>}else{g::<'static>};}
fn main() {}
