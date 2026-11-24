#[non_exhaustive]
pub struct OpenUnit;
pub use self::OpenUnit as Renamed;

#[non_exhaustive]
pub struct OpenTuple(pub u8);

#[non_exhaustive]
pub struct OpenRecord {
    pub value: u8,
}

#[non_exhaustive]
pub enum OpenEnum {
    First,
    Second,
}

pub fn unit() -> OpenUnit {
    OpenUnit
}

pub fn tuple(value: u8) -> OpenTuple {
    OpenTuple(value)
}

pub fn record(value: u8) -> OpenRecord {
    OpenRecord { value }
}

pub fn first() -> OpenEnum {
    OpenEnum::First
}
