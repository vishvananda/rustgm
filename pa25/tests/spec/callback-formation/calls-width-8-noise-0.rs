#![feature(const_trait_impl)]
const trait Factory:Sized{fn make()->Self;}
const fn invoke0<T:[const]Factory>()->T{T::make()}
const fn invoke1<T:[const]Factory>()->T{T::make()}
const fn invoke2<T:[const]Factory>()->T{T::make()}
const fn invoke3<T:[const]Factory>()->T{T::make()}
const fn invoke4<T:[const]Factory>()->T{T::make()}
const fn invoke5<T:[const]Factory>()->T{T::make()}
const fn invoke6<T:[const]Factory>()->T{T::make()}
const fn invoke7<T:[const]Factory>()->T{T::make()}
fn main(){}
