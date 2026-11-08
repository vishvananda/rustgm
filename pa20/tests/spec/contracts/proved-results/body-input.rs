trait Source{type Out;}
struct Short;
impl Source for Short{type Out=u8;}
fn pick<U:Source<Out=u8>>(){}
fn main(){pick();}
