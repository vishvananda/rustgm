#[path = "harness/native_references.rs"]
mod native_references;
const CASES: &[(&str, &str, bool)] = &[
    (
        "self-static",
        "struct N{v:u64,next:&'static N}static A:N=N{v:7,next:&A};fn main(){check(A.next.next.v==7);}",
        false,
    ),
    (
        "mutual-static",
        "struct N{v:u64,next:&'static N}static A:N=N{v:7,next:&B};static B:N=N{v:9,next:&A};fn main(){check(A.next.v==9 && B.next.v==7 && A.next.next.v==7);}",
        false,
    ),
    (
        "array-static-cycle",
        "struct N{all:&'static [N;2],v:u8}static A:[N;2]=[N{all:&A,v:3},N{all:&A,v:4}];const R:&u8=&A[1].v;fn main(){check(A[0].all[1].all[0].v==3 && *R==4);}",
        false,
    ),
    (
        "packed-static-cycle",
        "#[repr(C,packed)]struct N{v:u8,next:&'static N}static A:N=N{v:7,next:&A};fn main(){check(A.next.next.v==7);}",
        false,
    ),
    (
        "self-field-address",
        "struct S{v:u64,r:&'static u64}static A:S=S{v:42,r:&A.v};fn main(){check(*A.r==42);}",
        false,
    ),
    (
        "read-static-cycle",
        "struct N{v:u8,next:&'static N}static A:N=N{v:7,next:&B};static B:N=N{v:9,next:&A};const C:u8=A.next.next.v;const R:&u8=&A.next.v;fn main(){check(C==7 && *R==9);}",
        false,
    ),
    (
        "read-through-static-reference",
        "static N:u64=42;static P:&u64=&N;const C:u64=*P;fn main(){check(C==42 && *P==42);}",
        false,
    ),
    (
        "pattern-static-address",
        "static N:(u8,u8)=(7,9);const C:&u8=&N.1;fn f(x:&u8)->u8{match x{C=>1,_=>2}}fn main(){check(f(&9)==1 && f(&7)==2);}",
        false,
    ),
    (
        "return-mutable-field",
        "fn update<'a>(p:&'a mut(u64,u64),q:&(u64,u64))->&'a mut u64{let(a,b)=p;*a+=q.0;*b+=q.1;a}fn main(){let mut p=(1,2);let q=(3,4);let r=update(&mut p,&q);let n=*r;check(n+p.1==10);}",
        false,
    ),
    (
        "borrowed-nominal-constant",
        "#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:&S)->u8{match x{&C=>1,_=>2}}fn main(){check(f(&S(7))==1 && f(&S(9))==2);}",
        false,
    ),
    (
        "borrowed-product-constant",
        "const C:(u8,bool)=(7,true);fn f(x:&(u8,bool))->u8{match x{&C=>1,_=>2}}fn main(){check(f(&(7,true))==1 && f(&(9,true))==2);}",
        false,
    ),
    (
        "conditional-patterns",
        "fn f(x:&(u8,u8))->u8{let mut n=0;if let (a,b)=x && *a==1{n+=*a+*b;}let (a,b)=x else{loop{}};n+=*a+*b;while let (a,b)=x && true{n+=*a+*b;break;}n}fn main(){check(f(&(1,2))==9);}",
        false,
    ),
    (
        "constant-reference-pattern",
        "const C:&u8=&7;fn f(x:&u8)->u8{match x{C=>1,_=>2}}fn main(){check(f(&7)==1 && f(&9)==2);}",
        false,
    ),
    (
        "array-rest",
        "fn f(x:&[u8;3])->u8{let [a,tail@..]=x;*a+tail[0]+tail[1]}fn main(){check(f(&[1,2,3])==6);}",
        false,
    ),
    (
        "guard-zero-field",
        "fn f(x:&((),u8))->u8{match x{(z,n) if {let _=&*z;*n==7}=>*n,_=>0}}fn main(){check(f(&((),7))==7);}",
        false,
    ),
    (
        "untagged-enum",
        "enum E{A}fn f(x:&E)->u8{match x{E::A=>7}}fn main(){check(f(&E::A)==7);}",
        false,
    ),
    (
        "erased-reference-array",
        "struct S(&'static u8);fn main(){let a:[S;0]=[];let _=a;check(true);}",
        false,
    ),
    (
        "lifetime-struct",
        "struct S<'a>{r:&'a mut u8}fn inc(s:S<'_>){*s.r+=1}fn main(){let mut x=41;inc(S{r:&mut x});check(x==42);}",
        false,
    ),
    (
        "local-write",
        "fn add(x:&mut u64,y:&u64){*x+=*y}fn main(){let mut x=40;let y=2;add(&mut x,&y);check(x==42);}",
        false,
    ),
    (
        "return-reference",
        "fn id<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=42;check(*id(&x)==42);}",
        false,
    ),
    (
        "conditional-return",
        "fn pick<'a>(a:&'a u64,b:&'a u64,c:bool)->&'a u64{if c{a}else{b}}fn main(){let a=7;let b=9;check(*pick(&a,&b,true)==7 && *pick(&a,&b,false)==9);}",
        false,
    ),
    (
        "reborrow",
        "fn inc(x:&mut u64){*x+=1}fn main(){let mut x=40;let r=&mut x;inc(r);inc(r);check(*r==42);}",
        false,
    ),
    (
        "reference-product",
        "fn id<'a>(x:(&'a u8,&'a u8))->(&'a u8,&'a u8){x}fn main(){let a=7;let b=9;let r=id((&a,&b));check(*r.0==7 && *r.1==9);}",
        false,
    ),
    (
        "promoted-scalar",
        "fn f()->&'static u64{&(40+2)}fn main(){check(*f()==42);}",
        false,
    ),
    (
        "promoted-product",
        "fn f()->&'static (u64,[u8;3]){&(42,[7;3])}fn main(){let r=f();check(r.0==42 && r.1[2]==7);}",
        false,
    ),
    (
        "nested-promotion",
        "fn f()->&'static &'static u8{&&7}fn main(){check(**f()==7);}",
        false,
    ),
    (
        "promotion-effects",
        "fn side(x:&mut u64){*x+=1}fn f(x:&mut u64)->&'static u8{&{side(x);7}}fn main(){let mut n=0;check(*f(&mut n)==7 && n==1);}",
        false,
    ),
    (
        "empty-repeat-effects",
        "fn side(x:&mut u64)->u8{*x+=1;7}fn f(x:&mut u64)->&'static [u8;0]{&[side(x);0]}fn main(){let mut n=0;let r=f(&mut n);match r{[]=>()}check(n==1);}",
        false,
    ),
    (
        "static-projection",
        "static A:(u8,[u16;2])=(3,[7,11]);const R:&u16=&A.1[1];fn main(){check(*R==11 && A.1[0]==7);}",
        false,
    ),
    (
        "packed-reference",
        "static N:u64=42;#[repr(C,packed)]struct P{tag:u8,r:&'static u64}static A:P=P{tag:7,r:&N};fn main(){let r=A.r;check(*r==42 && A.tag==7);}",
        false,
    ),
    (
        "aligned-zero",
        "#[repr(align(64))]struct Z;fn f<'a>(z:&'a Z)->&'a Z{z}fn test(_: &Z)->u8{7}fn main(){let z=Z;let r=&z;check(test(f(r))==7);}",
        false,
    ),
    (
        "zero-field",
        "#[repr(align(64))]struct Z;struct S{n:u8,z:Z}fn test(_: &Z)->u8{7}fn main(){let s=S{n:3,z:Z};check(test(&s.z)==7 && s.n==3);}",
        false,
    ),
    (
        "function-item",
        "fn f()->u8{7}fn main(){let r=&f;check((*r)()==7);}",
        false,
    ),
    (
        "mutable-zero-promotion",
        "fn f()->&'static mut [u8;0]{&mut []}fn main(){match f(){[]=>()}}",
        false,
    ),
    (
        "mutable-temporary",
        "fn main(){let r=&mut 40u64;*r+=2;check(*r==42);}",
        false,
    ),
    (
        "mutable-reference-temporary",
        "fn main(){let a=7u8;let b=9u8;let rr=&mut &a;*rr=&b;check(**rr==9);}",
        false,
    ),
    (
        "call-temporary",
        "fn f()->u64{42}fn main(){let r=&f();if *r==42{check(*r==42)}else{check(false)}}",
        false,
    ),
    (
        "branch-temporary",
        "fn f(c:bool){let r=&(if c{7u64}else{9});check(*r==if c{7}else{9});}fn main(){f(true);f(false);}",
        false,
    ),
    (
        "pattern-product",
        "fn f(x:&(u8,(u8,u8)))->u8{let (a,(b,c))=x;*a+*b+*c}fn main(){check(f(&(1,(2,3)))==6);}",
        false,
    ),
    (
        "pattern-double-reference",
        "fn f(x:&&(u8,u8))->u8{match x{&&(a,b)=>a+b}}fn main(){check(f(&&(3,4))==7);}",
        false,
    ),
    (
        "guarded-alternatives",
        "fn f(x:&(u8,u8))->u8{let mut n=0;let v=match x{(a,_)|(_,a) if {n+=1;*a==2}=>*a,_=>0};n+v}fn main(){check(f(&(1,2))==4);}",
        false,
    ),
    (
        "mutable-pattern",
        "fn f(x:&mut (u8,u8)){let (a,b)=x;*a+=*b;}fn main(){let mut p=(3,4);f(&mut p);check(p.0==7 && p.1==4);}",
        false,
    ),
    (
        "higher-ranked-call",
        "fn f<'a>(x:&'a u8)->&'a u8{x}fn main(){let g:for<'a> fn(&'a u8)->&'a u8=f;let x=42;check(*g(&x)==42);}",
        false,
    ),
    (
        "last-use",
        "fn main(){let mut x=7u8;let r=&x;check(*r==7);x=9;check(x==9);}",
        false,
    ),
    (
        "wide-scalar",
        "fn f(x:&u128)->u128{*x}fn main(){let x=340282366920938463463374607431768211455u128;check(f(&x)==x);let b=true;check(*&b);let c='\u{1f980}';check(*&c==c);}",
        false,
    ),
    (
        "mutable-array",
        "fn main(){let a=&mut [1u8,2,3];a[1]+=5;check(a[0]==1 && a[1]==7 && a[2]==3);}",
        false,
    ),
    (
        "aligned-packed-field",
        "#[repr(C,packed(2))]struct P{a:u8,b:u16}fn f(x:&mut u16){*x+=1}fn main(){let mut p=P{a:3,b:41};f(&mut p.b);check(p.b==42 && p.a==3);}",
        false,
    ),
    (
        "promotion-overflow",
        "fn f()->&'static u8{&(255u8+1)}fn main(){let _=f();}",
        true,
    ),
    (
        "promoted-bounds",
        "fn f(i:usize)->&'static u8{&[1,2][i]}fn main(){let _=f(2);}",
        true,
    ),
];
#[test]
fn checked_thin_references_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native_references::check_cases("thin-references", CASES)
}
