fn main(){std::panic::set_hook(Box::new(|_|{}));let rhs=std::hint::black_box(8u8);let shift=std::panic::catch_unwind(||1u8<<rhs);println!("{}",shift.is_err());}
