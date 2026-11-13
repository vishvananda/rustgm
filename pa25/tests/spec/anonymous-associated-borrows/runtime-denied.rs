trait Value{const VALUE:usize;}impl Value for u8{const VALUE:usize=7;}
fn runtime()->usize{7}fn illegal()->usize{const{let p=&runtime();*p}}
