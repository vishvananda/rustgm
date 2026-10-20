fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait V{}struct S;impl V for S{}impl Drop for S{fn drop(&mut self){check(false);}}fn make()->impl V{S}fn consume<T>(_:T){}fn main(){consume(make());}
