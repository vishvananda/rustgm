fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait Make{fn make()->impl Value;}impl Make for u16{fn make()->impl Value{43u8}}fn run<T:Make>()->u8{T::make().value()}fn main(){check(run::<u16>()==43);}
