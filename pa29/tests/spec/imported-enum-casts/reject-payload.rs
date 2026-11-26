extern crate base;

pub fn rejected() -> u8 {
    base::Payload::Data(7) as u8
}
