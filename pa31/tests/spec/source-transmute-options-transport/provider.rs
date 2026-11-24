// Authored reduction of the pinned core mem/transmutability.rs Assume declaration.
#[lang="freeze"] pub unsafe auto trait Freeze {}
impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
#[unstable(feature = "transmutability", issue = "99571")]
#[lang = "transmute_opts"]
pub struct Assume {
    pub alignment: bool,
    pub lifetimes: bool,
    pub safety: bool,
    pub validity: bool,
}
