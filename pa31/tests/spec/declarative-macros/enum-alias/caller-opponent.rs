#![feature(decl_macro)]

macro define($name:ident, $alias:ident) {
    enum $name { Hidden }
    type $alias = $name;
}

define!(Choice, Alias);

fn main() {
    let _ = Alias::Hidden;
}
