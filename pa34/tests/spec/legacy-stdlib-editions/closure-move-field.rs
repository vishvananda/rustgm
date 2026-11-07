pub struct Token;
pub struct Pair { pub first: Token, pub second: Token }
pub fn split(pair: Pair) -> Token {
    let take = move || { let _first = pair.first; };
    take();
    pair.second
}
