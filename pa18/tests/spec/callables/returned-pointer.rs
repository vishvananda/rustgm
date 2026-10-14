fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<T>(x:T)->T{x}fn select<T>()->fn(T)->T{id::<T>}fn main(){let p=select::<u16>();check(p(300)==300);}
