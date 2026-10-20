//! Actual DST values qualify selected tail coercions independently through rustc.
#[path = "tail_coercions/mapped_tails.rs"]
mod mapped_tails;
#[path = "harness/native_references.rs"]
mod native;
#[path = "tail_coercions/persistent.rs"]
mod persistent;

const RECORD: &str = "struct S<T:?Sized>{tag:u16,tail:T}";

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn structural_unsizing_preserves_fields_metadata_and_reborrows()
-> Result<(), Box<dyn std::error::Error>> {
    let programs = [
        (
            "borrowed-literal",
            "fn read(s:&S<[u8]>)->u8{s.tail[1]}fn main(){check(read(&S{tag:7,tail:[3,5]})==5);}",
        ),
        (
            "borrowed-tuple-constructor",
            "struct W<T:?Sized>(u8,T);fn read(s:&W<[u8]>)->u8{s.1[1]}fn main(){check(read(&W(7,[3,5]))==5);}",
        ),
        (
            "borrowed-nested-literal",
            "struct W<T:?Sized>{inner:S<T>}fn read(s:&W<[u8]>)->u8{s.inner.tail[1]}fn main(){check(read(&W{inner:S{tag:7,tail:[3,5]}})==5);}",
        ),
        (
            "borrowed-empty-literal",
            "fn read(s:&S<[u8]>)->bool{match &s.tail{[]=>true,_=>false}}fn main(){check(read(&S{tag:7,tail:[]}));}",
        ),
        (
            "borrowed-callable-literal",
            "fn a()->u8{3}fn b()->u8{5}fn read(s:&S<[fn()->u8]>)->u8{s.tail[0]()+s.tail[1]()}fn main(){check(read(&S{tag:7,tail:[a,b]})==8);}",
        ),
        (
            "borrowed-generic-repeat",
            "fn read(s:&S<[u8]>)->u8{s.tail[1]}fn run<const N:usize>()->u8{read(&S{tag:7,tail:[5;N]})}fn main(){check(run::<2>()==5);}",
        ),
        (
            "shared",
            "fn read(s:&S<[u8]>)->u8{check(s.tag==7);match &s.tail{[a,b,c]=>*a+*b+*c,_=>0}}fn main(){let s=S{tag:7,tail:[1,3,9]};check(read(&s)==13);}",
        ),
        (
            "mutable",
            "fn edit(s:&mut S<[u8]>){s.tail[1]=17;s.tag=9;}fn main(){let mut s=S{tag:7,tail:[1,3,9]};let a=&mut s;edit(a);check(a.tail[1]==17);check(s.tag==9);}",
        ),
        (
            "nested",
            "struct W<T:?Sized>{prefix:u32,inner:S<T>}fn read(s:&W<[u8]>)->u8{check(s.prefix==31);check(s.inner.tag==7);s.inner.tail[2]}fn main(){let s=W{prefix:31,inner:S{tag:7,tail:[1,3,9]}};check(read(&s)==9);}",
        ),
        (
            "empty",
            "fn read(s:&S<[u8]>)->bool{check(s.tag==7);match &s.tail{[]=>true,_=>false}}fn main(){let s=S{tag:7,tail:[]};check(read(&s));}",
        ),
        (
            "zst",
            "fn read(s:&S<[()]>)->bool{match &s.tail{[(),(),()]=>true,_=>false}}fn main(){let s=S{tag:7,tail:[();3]};check(read(&s));}",
        ),
        (
            "generic",
            "fn convert<T,const N:usize>(s:&S<[T;N]>)->&S<[T]>{s}fn main(){let s=S{tag:7,tail:[5u8,9]};let p=convert(&s);check(p.tail[1]==9);}",
        ),
        (
            "branch",
            "fn choose<'a>(ok:bool,a:&'a S<[u8;2]>,b:&'a S<[u8;3]>)->&'a S<[u8]>{if ok{a}else{b}}fn main(){let a=S{tag:7,tail:[3,5]};let b=S{tag:9,tail:[7,11,13]};check(choose(false,&a,&b).tail[2]==13);}",
        ),
        (
            "match",
            "fn choose<'a>(ok:bool,a:&'a S<[u8;2]>,b:&'a S<[u8;3]>)->&'a S<[u8]>{match ok{true=>a,false=>b}}fn main(){let a=S{tag:7,tail:[3,5]};let b=S{tag:9,tail:[7,11,13]};check(choose(true,&a,&b).tail[1]==5);}",
        ),
        (
            "assignment",
            "fn main(){let a=S{tag:7,tail:[3u8,5]};let b=S{tag:9,tail:[7,11,13]};let mut p:&S<[u8]>=&a;p=&b;check(p.tail[2]==13);}",
        ),
        (
            "aggregate",
            "struct R<'a>{value:&'a S<[u8]>}fn main(){let a=S{tag:7,tail:[3u8,5]};let r=R{value:&a};check(r.value.tail[1]==5);}",
        ),
        (
            "explicit-dereference",
            "fn read(s:&S<[u8]>)->u8{s.tail[1]}fn main(){let a=S{tag:7,tail:[3,5]};check(read(*(&&a))==5);}",
        ),
        (
            "borrow-tail",
            "fn read(s:&S<[u8]>)->&[u8]{&s.tail}fn main(){let a=S{tag:7,tail:[3,5]};let p=read(&a);check(p[1]==5);}",
        ),
    ];
    let programs = programs.map(|(name, source)| (name, format!("{RECORD}{source}")));
    let cases = programs
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("pa22-native/tail-coercions", &cases)
}
