pub trait Schema {
    type Data;
}

pub struct Concrete;
impl Schema for Concrete {
    type Data = u8;
}

pub struct Record<T: Schema> {
    pub data: T::Data,
}

pub fn build<T: Schema<Data = u8>>(data: u8) -> Record<T> {
    Record { data }
}

pub fn build_concrete(data: u8) -> Record<Concrete> {
    Record { data }
}
