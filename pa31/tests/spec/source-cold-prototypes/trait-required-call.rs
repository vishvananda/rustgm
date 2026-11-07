trait Owner{#[cold]fn prototype()->u8;}impl Owner for u8{fn prototype()->u8{23}}pub fn observed<T:Owner>()->u8{T::prototype()}pub fn invoke()->u8{observed::<u8>()}
