trait B<'a>{}trait A{type G<'a>where Self:B<'a>;}fn item<T:A+for<'a>B<'a>>(){let _:for<'a>fn(T::G<'a>);}fn main(){}
