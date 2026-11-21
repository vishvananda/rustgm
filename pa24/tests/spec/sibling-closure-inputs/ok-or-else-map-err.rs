fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Code{fn code(self)->u32;}
impl<'a> Code for &'a str{fn code(self)->u32{3}}
struct E(u32);
fn at(site:u32,m:impl Code)->E{E(site+m.code())}
enum Opt<T>{Some(T),None}
enum Res<T,X>{Ok(T),Err(X)}
impl<T> Opt<T>{fn ok_or_else<X,F:FnOnce()->X>(self,f:F)->Res<T,X>{match self{Opt::Some(t)=>Res::Ok(t),Opt::None=>Res::Err(f())}}}
impl<T,X> Res<T,X>{fn map_err<Y,F:FnOnce(X)->Y>(self,f:F)->Res<T,Y>{match self{Res::Ok(t)=>Res::Ok(t),Res::Err(x)=>Res::Err(f(x))}}}
fn parse(n:u8)->Res<u8,&'static str>{if n==0{Res::Err("zero")}else{Res::Ok(n)}}
fn get(o:Opt<u8>)->Res<u8,E>{let error=|message|at(10,message);let n=match o.ok_or_else(||error("missing")){Res::Ok(n)=>n,Res::Err(e)=>return Res::Err(e)};parse(n).map_err(error)}
fn main(){check(match get(Opt::None){Res::Err(E(13))=>true,_=>false});check(match get(Opt::Some(0)){Res::Err(E(13))=>true,_=>false});check(match get(Opt::Some(4)){Res::Ok(4)=>true,_=>false});}
