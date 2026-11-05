trait Lending{type Ref<'a>where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>;}fn pair<'a,'b,F:Lending+'a+'b>(a:&'a F,b:&'b F)->(F::Ref<'a>,F::Ref<'b>){(F::get(a),F::get(b))}fn main(){}
