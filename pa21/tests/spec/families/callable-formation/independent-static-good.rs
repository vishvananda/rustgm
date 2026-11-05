trait A{type G<'a> where Self:'static;fn get<'a>(&'a self)->Self::G<'a> where Self:'static;}fn item<T:A+'static>(){let _=T::get;}fn main(){}
