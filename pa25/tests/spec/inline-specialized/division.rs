fn f<const N:u8>()->u8{const{1/N}}pub fn run()->bool{let _=f::<1>();let _=f::<0>();true}
