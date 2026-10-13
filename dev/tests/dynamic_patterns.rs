#[path = "harness/native_references.rs"]
mod native_references;

const CASES: &[(&str, &str, bool)] = &[
    (
        "lengths",
        "fn f(s:&[u8])->u8{match s{[]=>0,[x]=>*x,[x,y]=>*x+*y,[x,..,y]=>*x+*y}}fn main(){check(f(&[])==0 && f(&[7])==7 && f(&[3,4])==7 && f(&[2,1,8])==10);}",
        false,
    ),
    (
        "prefix-suffix",
        "fn f(s:&[u8])->u8{match s{[1,..,3]=>7,[1,..]=>8,[..,3]=>9,_=>0}}fn main(){check(f(&[1,3])==7 && f(&[1])==8 && f(&[3])==9 && f(&[])==0 && f(&[0,1,3])==9);}",
        false,
    ),
    (
        "rest-return",
        "fn f(s:&[u8])->&[u8]{match s{[_,mid @ ..,_]=>mid,_=>s}}fn main(){check(f(&[1,7,9,2])[1]==9 && f(&[7])[0]==7);let _=f(&[1,2]);}",
        false,
    ),
    (
        "mutable-split",
        "fn f(s:&mut [u8]){if let [a,mid @ ..,b]=s{*a+=1;mid[0]+=2;*b+=3;}}fn main(){let mut a=[1,2,3,4];f(&mut a);check(a[0]==2 && a[1]==4 && a[2]==3 && a[3]==7);}",
        false,
    ),
    (
        "mutable-empty-rest",
        "fn f(s:&mut [u8]){if let [a,mid @ ..,b]=s{*a+=1;let _=mid;*b+=3;}}fn main(){let mut a=[1,4];f(&mut a);check(a[0]==2 && a[1]==7);}",
        false,
    ),
    (
        "multiple-suffix",
        "fn f(s:&mut [u8]){if let [..,a,b]=s{*a+=1;*b+=2;}}fn main(){let mut a=[1,2,3];f(&mut a);check(a[0]==1 && a[1]==3 && a[2]==5);}",
        false,
    ),
    (
        "rest-loop",
        "fn f(mut s:&[u8])->u64{let mut n=0;while let [x,rest @ ..]=s{n+=*x as u64;s=rest;}n}fn main(){check(f(&[3,7,9])==19 && f(&[])==0);}",
        false,
    ),
    (
        "rest-let-else",
        "fn f(s:&[u8])->u8{let [a,rest @ ..]=s else{return 0;};let [_,b]=rest else{return *a;};*a+*b}fn main(){check(f(&[])==0 && f(&[3])==3 && f(&[3,7,9])==12);}",
        false,
    ),
    (
        "whole-rest",
        "fn f(s:&[u8])->&[u8]{let [rest @ ..]=s;rest}fn main(){check(f(&[7,9])[1]==9);let _=f(&[]);}",
        false,
    ),
    (
        "nested-slice",
        "fn f(s:&[&[u8]])->u8{match s{[[],..,[_,v]]=>*v,[[x,..],..]=>*x,_=>0}}fn main(){let a:&[u8]=&[];let b:&[u8]=&[7,9];check(f(&[a,b])==9 && f(&[b,a])==7 && f(&[a])==0);}",
        false,
    ),
    (
        "slice-guard-retry",
        "fn f(s:&[u8])->u8{let mut calls=0;match s{[x,..]|[..,x] if {calls+=1;*x==9}=>calls,_=>calls+10}}fn main(){check(f(&[2,9])==2 && f(&[9,2])==1 && f(&[2,3])==12 && f(&[])==10);}",
        false,
    ),
    (
        "slice-rest-guard",
        "fn f(s:&[u8])->u8{match s{[_,rest @ ..]|[rest @ ..,_] if rest[0]==7=>rest[1],_=>0}}fn main(){check(f(&[1,7,9])==9 && f(&[7,9,1])==9);}",
        false,
    ),
    (
        "mutable-guard",
        "fn f(s:&mut [u8]){match s{[a,mid @ ..,b] if *a==1 && *b==4=>{*a=2;mid[0]=9;*b=5;},_=>()}}fn main(){let mut a=[1,2,4];f(&mut a);check(a[0]==2 && a[1]==9 && a[2]==5);}",
        false,
    ),
    (
        "zero-sized",
        "#[derive(Clone,Copy)]struct Z;fn f(s:&[Z])->u8{match s{[]=>0,[Z]=>1,[Z,..,Z]=>2}}fn main(){check(f(&[])==0 && f(&[Z])==1 && f(&[Z;4])==2);}",
        false,
    ),
    (
        "noncopy-borrow",
        "struct N(u8);fn f(s:&[N])->u8{match s{[a,..,b]=>a.0+b.0,_=>0}}fn main(){check(f(&[N(7),N(9)])==16);}",
        false,
    ),
    (
        "byte-patterns",
        "const C:&[u8]=b\"abc\";fn f(s:&[u8])->u8{match s{C=>1,b\"\"=>2,b\"a\"=>3,b\"\\0\\xff\"=>4,_=>0}}fn main(){check(f(b\"abc\")==1 && f(b\"\")==2 && f(b\"a\")==3 && f(b\"\\0\\xff\")==4 && f(b\"abd\")==0);}",
        false,
    ),
    (
        "text-bytes",
        "fn f(s:&str)->u8{match s{\"\"=>1,\"a\"=>2,\"ab\"=>3,\"a\\0b\"=>4,\"héllo\"=>5,\"🦀\"=>6,\"e\\u{301}\"=>7,_=>0}}fn main(){check(f(\"\")==1 && f(\"a\")==2 && f(\"ab\")==3 && f(\"a\\0b\")==4 && f(\"héllo\")==5 && f(\"🦀\")==6 && f(\"e\\u{301}\")==7 && f(\"é\")==0 && f(\"ac\")==0 && f(\"abc\")==0 && f(\"a\\0c\")==0);}",
        false,
    ),
    (
        "text-constant",
        "const A:&str=\"héllo\";const R:&&str=&&\"🦀\";fn f(s:&str)->u8{match s{A=>7,_=>0}}fn g(s:&&str)->u8{match s{R=>9,_=>0}}fn main(){check(f(A)==7 && f(\"hello\")==0 && g(&\"🦀\")==9 && g(&\"héllo\")==0);}",
        false,
    ),
    (
        "text-guard",
        "fn f(s:&str,b:bool)->u8{match s{\"héllo\" if b=>7,\"héllo\"=>9,\"\"=>1,_=>0}}fn main(){check(f(\"héllo\",true)==7 && f(\"héllo\",false)==9 && f(\"\",false)==1 && f(\"x\",true)==0);}",
        false,
    ),
    (
        "text-conditional",
        "fn f(s:&str)->u8{if let \"a\"=s{return 1;}let \"b\"=s else{return 0;};2}fn main(){check(f(\"a\")==1 && f(\"b\")==2 && f(\"c\")==0);}",
        false,
    ),
    (
        "text-slices",
        "fn f(s:&[&str])->u8{match s{[\"a\",..,\"🦀\"]=>7,[..,\"b\"]=>9,_=>0}}fn main(){check(f(&[\"a\",\"héllo\",\"🦀\"])==7 && f(&[\"b\"])==9 && f(&[])==0);}",
        false,
    ),
    (
        "packed-text",
        "#[repr(C,packed)]struct S{tag:u8,s:&'static str}static V:S=S{tag:3,s:\"héllo\"};fn main(){let n=match V.s{\"héllo\"=>7,_=>0};check(n==7 && V.tag==3);}",
        false,
    ),
    (
        "packed-slice",
        "#[repr(C,packed)]struct S{tag:u8,s:&'static [u8]}static V:S=S{tag:3,s:&[7,9]};fn main(){let n=match V.s{[a,..,b]=>*a+*b,_=>0};check(n==16 && V.tag==3);}",
        false,
    ),
];

#[test]
fn dynamic_slice_and_text_patterns_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native_references::check_cases("dynamic-patterns", CASES)
}

#[test]
fn long_text_compares_all_bytes_without_expanding_comparison_code()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = "a".repeat(32_768);
    let source = format!(
        "fn f(s:&str)->bool{{match s{{\"{prefix}b\"=>true,_=>false}}}}fn main(){{check(f(\"{prefix}b\") && !f(\"{prefix}c\") && !f(\"{prefix}\"));}}"
    );
    native_references::check_cases("long-text-pattern", &[("long", &source, false)])?;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/long-text-pattern-{}/long.rs",
        std::process::id()
    ));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--count-lowir")
        .arg(path)
        .output()?;
    assert!(output.status.success() && output.stderr.is_empty());
    let text = String::from_utf8(output.stdout)?;
    let instructions = text
        .split_whitespace()
        .skip_while(|word| *word != "instructions")
        .nth(1)
        .ok_or("instruction count")?
        .parse::<u64>()?;
    assert!(instructions < 200, "{text}");
    Ok(())
}
