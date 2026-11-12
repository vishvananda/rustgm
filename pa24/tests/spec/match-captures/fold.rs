fn make<T,B,Acc>(mut f:impl FnMut(T)->Option<B>,mut fold:impl FnMut(Acc,B)->Acc)->impl FnMut(Acc,T)->Acc{move|acc,item|match f(item){Some(x)=>fold(acc,x),None=>acc}}
