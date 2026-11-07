pub struct Token;
pub fn keep(token: Token) -> Token {
    let observe = || { let _ = token; };
    let moved = token;
    observe();
    moved
}
