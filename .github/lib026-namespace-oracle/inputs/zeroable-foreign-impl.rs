extern crate base; #[repr(transparent)] #[derive(Clone,Copy)] struct Local(u8); unsafe impl base::num::ZeroablePrimitive for Local {type NonZeroInner=core::num::NonZeroU8;}
