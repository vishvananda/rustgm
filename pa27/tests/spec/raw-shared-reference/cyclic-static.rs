struct S{r:&'static S}static VALUE:S=S{r:unsafe{&*(&raw const VALUE)}};fn main(){let _=&VALUE;}
