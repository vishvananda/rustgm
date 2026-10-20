trait A{}struct H<T:?Sized+'static>{r:*const T}type P=H<dyn A>;fn main(){}
