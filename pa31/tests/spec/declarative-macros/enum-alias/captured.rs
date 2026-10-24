#![feature(decl_macro)]

macro define($name:ident, $alias:ident, $variant:ident, $function:ident) {
    enum $name { $variant }
    type $alias = $name;
    fn $function() -> u8 {
        let _ = $alias::$variant;
        7
    }
}

define!(Choice, Alias, First, answer);

fn main() {
    println!("{}", answer());
}
