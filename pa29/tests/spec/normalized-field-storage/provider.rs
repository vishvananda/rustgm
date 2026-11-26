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

pub fn read(instance: &Record<Concrete>) -> u8 {
    instance.data
}

pub fn replace(instance: &mut Record<Concrete>, value: u8) {
    instance.data = value;
}

pub struct Bytes;

impl Schema for Bytes {
    type Data = [u8; 2];
}

pub fn bytes(instance: &Record<Bytes>) -> [u8; 2] {
    instance.data
}
