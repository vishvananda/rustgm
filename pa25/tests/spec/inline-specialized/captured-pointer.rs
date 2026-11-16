fn ident<T>(x:T)->T{x}const fn pick<T>()->fn(T)->T{ident::<T>}fn apply<T>(x:T)->T{let f=const{let g=pick::<T>();move|v:T|g(v)};f(x)}pub fn run()->bool{apply(41u8)==41&&apply(7u16)==7}
