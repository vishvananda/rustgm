trait A{type Out;}trait B<P>{}trait C{type N;}trait X<T:A<Out:B<<T::Out as C>::N>>>{}fn main(){}
