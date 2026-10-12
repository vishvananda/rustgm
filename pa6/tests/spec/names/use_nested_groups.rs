mod a{pub mod b{pub struct X;pub struct Y;}}use a::{b::{X,Y}};fn f(){X;Y;}
