fn ident<T>(x:T)->T{x}const fn pick<T>()->fn(T)->T{ident::<T>}fn apply<T>(x:T)->T{(const{pick::<T>()})(x)}pub fn run()->bool{apply(41u8)==41&&apply(7u16)==7}
