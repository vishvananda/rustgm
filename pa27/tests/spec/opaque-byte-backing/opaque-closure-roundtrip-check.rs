const fn make()->impl Fn()->u8+Copy {||61}
union U<T:Copy>{a:T,b:T}
const fn round<T:Copy>(x:T)->T{let u=U{a:x};unsafe{u.b}}
fn main(){let x=const{round(make())};if x()!=61{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
