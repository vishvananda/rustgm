#[derive(Clone)] struct N(u8); #[derive(Clone)] #[repr(packed(2))] struct S(N);
