fn g<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'a>(x:&'a u8,c:bool)->&'a u8{let p=if c{g::<'a>}else{g::<'a>};p(x)}
fn main() {}
