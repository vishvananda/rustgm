#[derive(Clone, Copy)]
union MaybeEscapedCharacter<const N: usize> {
    pub escape_seq: [ascii::Char; N],
    pub literal: char,
}
