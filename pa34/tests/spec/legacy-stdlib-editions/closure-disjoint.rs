pub struct Pair { pub first: u8, pub second: u8 }
pub fn bump(pair: &mut Pair) -> u8 {
    let mut add = || pair.first += 1;
    let second = &pair.second;
    add();
    *second
}
