fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Wrap<T>(T);fn wrap<T,F:FnOnce()->T>(f:F)->impl FnOnce()->Wrap<T>{move||Wrap(f())}fn project<T,F:FnOnce()->T>(f:F)->T{wrap(f)().0}
fn main(){check(project(||23u8)==23);}
