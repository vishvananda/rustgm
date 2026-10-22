fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K{}impl K for u8{}fn first<T:K>(_:T){}fn main(){first(1u8);check([1u8]<[2u8]);}