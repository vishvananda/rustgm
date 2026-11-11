fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Wrap<T>(T);trait Make{type Output;fn make(self)->Self::Output;}struct Factory<T>(T);impl<T>Make for Factory<T>{type Output=Wrap<T>;fn make(self)->Wrap<T>{Wrap(self.0)}}fn wrap<T>(v:T)->impl Make<Output=Wrap<T>>{Factory(v)}fn drive<R,F:Make<Output=R>>(f:F)->R{f.make()}fn project<T>(v:T)->T{drive(wrap(v)).0}
fn main(){check(project(23u8)==23);}
