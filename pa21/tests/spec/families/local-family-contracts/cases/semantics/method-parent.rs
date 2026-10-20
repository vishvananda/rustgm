trait Q<P>{type Out<T>;fn f<T>(p:P,t:T)->Self::Out<T>;}impl<P>Q<P> for u8{type Out<T>=(P,T);fn f<U>(p:P,t:U)->(P,U){(p,t)}}
fn main(){}
