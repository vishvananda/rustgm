enum Maybe<T>{Some(T),None}
unsafe extern "C"{fn record(tag:u8,value:u8);}
fn make(mut f:impl FnMut(u8)->Maybe<u8>,mut fold:impl FnMut(u8,u8)->u8)->impl FnMut(u8,u8)->u8{move|acc,item|match f(item){Maybe::Some(x)=>fold(acc,x),Maybe::None=>acc}}
#[unsafe(no_mangle)]pub extern "C" fn code(value:u8)->u8{
 let mut run=make(move|item|{unsafe{record(1,item)};match item{0=>Maybe::None,_=>Maybe::Some(value)}},|_acc,x|{unsafe{record(2,x)};x});
 let first=run(3,1);unsafe{record(3,first)};
 let second=run(9,0);unsafe{record(4,second)};second
}
