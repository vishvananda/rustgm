#[lang = "sync"]
#[stable(feature = "alias_api", since = "1.0.0")]
pub unsafe auto trait Sync {}

#[lang = "freeze"]
#[stable(feature = "alias_api", since = "1.0.0")]
pub unsafe auto trait Freeze {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
#[unstable(feature = "role_fixture", issue = "none")]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}

mod inner {
    pub(crate) type Word = u32;
    pub(crate) type Pair = (u32, u32);
}

use inner::Word;
pub(crate) use inner::Pair;

mod hidden {
    use super::inner::*;

    pub fn second(pair: Pair) -> Word {
        pair.1
    }
}

#[stable(feature = "alias_api", since = "1.0.0")]
pub type PublicWord = Word;

#[stable(feature = "alias_api", since = "1.0.0")]
pub static BASE_WORD: PublicWord = 19;

#[stable(feature = "alias_api", since = "1.0.0")]
pub fn read_base() -> Word {
    BASE_WORD
}

#[stable(feature = "alias_api", since = "1.0.0")]
pub fn pair_first(pair: Pair) -> PublicWord {
    pair.0
}

#[stable(feature = "alias_api", since = "1.0.0")]
pub fn pair_second(pair: (u32, u32)) -> u32 {
    hidden::second(pair)
}
