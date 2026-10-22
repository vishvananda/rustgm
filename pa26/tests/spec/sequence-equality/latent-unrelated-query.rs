fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K{}impl K for u8{}fn earlier<T:K>(_:T){}fn later()->bool{[1u8]!=[2u8]}fn main(){earlier(3u8);check(later())}
