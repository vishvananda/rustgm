#[path = "harness/native_references.rs"]
mod native_references;

const CASES: &[(&str, &str, bool)] = &[
    (
        "slice-guard",
        "fn f(s:&[u8])->u8{match s{t if t[0]==7=>t[1],_=>0}}fn main(){check(f(&[7,9])==9 && f(&[8,9])==0);}",
        false,
    ),
    (
        "slice-branch",
        "fn f<'a>(a:&'a [u8],b:&'a [u8],c:bool)->&'a [u8]{if c{a}else{b}}fn main(){let a=[1];let b=[2,3];check(f(&a,&b,false)[1]==3 && f(&a,&b,true)[0]==1);}",
        false,
    ),
    (
        "slice-empty",
        "fn id(s:&[u8])->&[u8]{s}fn main(){let _=id(&[]);let _=id(b\"\");}",
        false,
    ),
    (
        "slice-mutable-temporary",
        "fn f(s:&mut [u8]){s[1]+=1}fn main(){let a:&mut [u8]=&mut [7,8];f(a);check(a[1]==9);}",
        false,
    ),
    (
        "text-reborrow",
        "const S:&str=&*\"héllo\";fn id(s:&str)->&str{s}fn main(){let _=id(S);let _=id(&*\"🦀\");}",
        false,
    ),
    (
        "slice-parameter",
        "fn get(x:&[u8],i:usize)->u8{x[i]}fn main(){check(get(&[3,7,9],1)==7);}",
        false,
    ),
    (
        "slice-return",
        "fn id<'a>(x:&'a [u8])->&'a [u8]{x}fn main(){let a=[3,7,9];let s=id(&a);check(s[2]==9);}",
        false,
    ),
    (
        "slice-mutable",
        "fn inc(x:&mut [u8]){x[1]+=1}fn main(){let mut a=[1,41,3];inc(&mut a);check(a[1]==42);}",
        false,
    ),
    (
        "slice-reborrow",
        "fn inc(x:&mut [u8]){x[0]+=1}fn main(){let mut a=[40];let s:&mut [u8]=&mut a;inc(s);inc(s);check(s[0]==42);}",
        false,
    ),
    (
        "slice-promoted",
        "fn f()->&'static [u8]{&[7,9]}fn main(){check(f()[1]==9);}",
        false,
    ),
    (
        "slice-constant",
        "const S:&[u8]=&[7,9];fn main(){check(S[0]==7 && S[1]==9);}",
        false,
    ),
    (
        "slice-static",
        "static A:[u8;2]=[7,9];static S:&[u8]=&A;fn main(){check(S[0]==7 && S[1]==9);}",
        false,
    ),
    (
        "slice-product",
        "struct S<'a>{x:&'a [u8],y:u64}fn f(x:S<'_>)->S<'_>{x}fn main(){let a=[7,9];let s=f(S{x:&a,y:42});check(s.x[1]==9 && s.y==42);}",
        false,
    ),
    (
        "slice-packed",
        "#[repr(C,packed)]struct S{x:u8,p:&'static [u8]}static A:S=S{x:3,p:&[7,9]};fn main(){let p=A.p;check(p[1]==9 && A.x==3);}",
        false,
    ),
    (
        "slice-zero-elements",
        "#[derive(Clone,Copy)]struct Z;fn f(s:&[Z]){let _=&s[2];}fn main(){f(&[Z;3]);}",
        false,
    ),
    (
        "slice-overaligned-zero",
        "#[repr(align(64))]#[derive(Clone,Copy)]struct Z;fn f(s:&[Z]){let _=&s[2];}fn main(){f(&[Z;3]);}",
        false,
    ),
    (
        "slice-borrow-element",
        "fn get<'a>(s:&'a [u8])->&'a u8{&s[1]}fn main(){let a=[7,9];check(*get(&a)==9);}",
        false,
    ),
    (
        "bytes-literal",
        "fn f(s:&[u8])->u8{s[2]}fn main(){check(f(b\"abc\")==99);}",
        false,
    ),
    (
        "bytes-constant",
        "const S:&[u8]=b\"abc\";fn main(){check(S[1]==98);}",
        false,
    ),
    (
        "text-values",
        "const S:&str=\"héllo\";fn id<'a>(s:&'a str)->&'a str{s}fn main(){let a=id(S);let b=id(\"🦀\");let _=(a,b);check(true);}",
        false,
    ),
    (
        "nested-text",
        "const R:&&str=&&\"🦀\";fn id<'a>(s:&'a str)->&'a str{s}fn main(){let _=id(*R);}",
        false,
    ),
    (
        "slice-bounds",
        "fn f(s:&[u8],i:usize)->u8{s[i]}fn main(){let _=f(&[7,9],2);}",
        true,
    ),
    (
        "slice-empty-bounds",
        "fn f(s:&[u8],i:usize)->u8{s[i]}fn main(){let _=f(&[],0);}",
        true,
    ),
    (
        "slice-zero-bounds",
        "#[derive(Clone,Copy)]struct Z;fn f(s:&[Z]){let _=&s[3];}fn main(){f(&[Z;3]);}",
        true,
    ),
];
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn wide_references_execute_with_pointer_and_length() -> Result<(), Box<dyn std::error::Error>> {
    native_references::check_cases("wide-references", CASES)
}
