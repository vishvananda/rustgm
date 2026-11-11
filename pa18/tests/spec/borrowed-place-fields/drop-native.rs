fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Wrap<T>(T);impl<T>Wrap<T>{fn get(&self)->u8{23}fn change(&mut self)->u8{41}}
struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}
fn probe<T>(w:&Wrap<T>)->u8{Wrap::<T>::get(w)}
fn main(){let mut count=0u8;{let w=Wrap(D(&mut count));check(probe(&w)==23);}check(count==1);}
