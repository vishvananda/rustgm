#![feature(decl_macro)]

macro define($name:ident, $alias:ident, $function:ident) {
    enum $name { Hidden }
    type $alias = $name;
    fn $function() -> u8 {
        let _ = $alias::Hidden;
        7
    }
}

define!(Choice, Alias, answer);

fn main() {
    println!("{}", answer());
}
