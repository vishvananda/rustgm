use std::mem::size_of_val;
fn main(){let rhs=1;let encoded=3usize<<rhs|1;println!("{} {}",size_of_val(&rhs),encoded);}
