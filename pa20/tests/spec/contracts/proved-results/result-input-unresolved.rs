trait Source{type Out;}
fn pick<V:Source,I:Source<Out=<V as Source>::Out>>(){}
fn call<I:Source>(){pick::<_,I>();}
struct Short;
impl Source for Short{type Out=u8;}
fn main(){call::<Short>();}
