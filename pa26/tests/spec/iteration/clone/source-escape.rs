struct V(u8);fn bad<'a>()->core::slice::Iter<'a,V>{let a=[V(1)];a.iter().clone()}fn main(){}
