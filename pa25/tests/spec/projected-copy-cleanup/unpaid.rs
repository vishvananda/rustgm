trait Has{type Meta;}const fn consume<T:Has>(x:T::Meta){}fn main(){}
