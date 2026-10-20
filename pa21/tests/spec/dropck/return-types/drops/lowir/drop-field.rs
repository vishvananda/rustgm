fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait V{}struct S;impl V for S{}impl Drop for S{fn drop(&mut self){check(false);}}struct W<T>(T);fn make()->impl V{S}fn main(){let _x=W(make());}
