struct A<T>(fn(T));struct B<T>(fn(T));struct C<T>(A<T>,B<T>);impl<T> Unpin for A<T> where B<T>:Unpin,T:Sync{}impl<T> Unpin for B<T> where A<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<C<u8>>();}
fn main(){}
