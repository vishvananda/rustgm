use std::{
    path::{Path, PathBuf},
    process::Command,
};
const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "contract-scalar-retry",
        "fn entry()->u64{let mut c=0;match 1u8{1|_ if {c+=1;false}=>99,_=>c}}\nfn main(){check(entry()==2);}",
    ),
    (
        "contract-product-retry",
        "fn entry()->u64{let mut c=0;match (0u8,0u8){(0|_,0|_) if {c+=1;false}=>99,_=>c}}\nfn main(){check(entry()==4);}",
    ),
    (
        "contract-triple-retry",
        "fn entry()->u64{let mut c=0;match (0u8,0u8,0u8){(0|_,0|_,0|_) if {c+=1;false}=>99,_=>c}}\nfn main(){check(entry()==8);}",
    ),
    (
        "contract-overlap-retry",
        "fn entry()->u64{let mut c=0;match 7u8{0..=9|5..=15|_ if {c+=1;false}=>99,_=>c}}\nfn main(){check(entry()==3);}",
    ),
    (
        "contract-scrutinee-once",
        "fn entry()->u64{let mut c=0;match {c+=100;1u8}{1|_ if {c+=1;false}=>99,_=>c}}\nfn main(){check(entry()==102);}",
    ),
    (
        "contract-disjoint-write",
        "fn entry()->u64{let mut x=(7u8,1u8);match x{(n,_) if {x.1+=1;n==0}=>99,_=>x.1 as u64}}\nfn main(){check(entry()==2);}",
    ),
    (
        "contract-guard-field-copy",
        "struct N(u8);fn entry()->u64{let x=N(7);match x{n if n.0==7=>n.0 as u64,_=>2}}\nfn main(){check(entry()==7);}",
    ),
    (
        "contract-conditional-guard",
        "fn entry()->u64{match (0u8,7u8){x if let (0,n)=x=>n as u64,_=>99}}\nfn main(){check(entry()==7);}",
    ),
    (
        "contract-conditional-guard-chain",
        "fn entry()->u64{match (0u8,7u8){x if let (0,n)=x&&n==7=>n as u64,_=>99}}\nfn main(){check(entry()==7);}",
    ),
    (
        "contract-conditional-shadow",
        "fn entry()->u64{match 7u8{n if let (0,n)=(0u8,n+1)=>n as u64,_=>99}}\nfn main(){check(entry()==8);}",
    ),
    (
        "contract-guard-return",
        "fn entry()->u64{let mut c=0;match 1u8{1|_ if {c+=1;return c;}=>99,_=>c}}\nfn main(){check(entry()==1);}",
    ),
    (
        "contract-uninitialized-wildcard",
        "fn entry()->u64{let x:u8;match x{_ if true=>1,_=>2}}\nfn main(){check(entry()==1);}",
    ),
    (
        "places-wildcard-product-field",
        "fn entry()->u64{let mut x=(7u8,1u8);match x{(_,_) if {x.1=2;false}=>99,_=>x.1 as u64}}fn main(){check(entry()==2);}",
    ),
    (
        "places-wildcard-whole",
        "fn entry()->u64{let mut x=(7u8,1u8);match x{_ if {x=(8,2);false}=>99,_=>x.1 as u64}}fn main(){check(entry()==2);}",
    ),
    (
        "places-disjoint-move",
        "struct N(u8);fn entry()->u64{let x=(7u8,N(1));match x{(n,_) if {let v=x.1;let _=v;n==0}=>99,_=>x.0 as u64}}fn main(){check(entry()==7);}",
    ),
    (
        "places-guard-local-mut",
        "fn entry()->u64{match 7u8{_ if let mut n=7u8&&{n+=1;true}=>n as u64,_=>99}}fn main(){check(entry()==8);}",
    ),
    (
        "places-guard-local-move",
        "struct N(u8);fn entry()->u64{let y=N(7);match 7u8{_ if let n=y&&true=>n.0 as u64,_=>99}}fn main(){check(entry()==7);}",
    ),
    (
        "flow-future-capture",
        "fn entry()->u64{let mut x=(7u8,1u8);match x{_ if {x.1=2;false}=>99,(_,n)=>n as u64}}fn main(){check(entry()==2);}",
    ),
    (
        "flow-uninitialized-wildcard-return",
        "fn entry()->u64{let x:u8;match x{_ if return 7=>99,_=>0}}fn main(){check(entry()==7);}",
    ),
    (
        "flow-guard-mutates-tested-full-range",
        "fn entry()->u64{let mut x=7u8;match x{0..=255 if {x=8;false}=>99,_=>x as u64}}fn main(){check(entry()==8);}",
    ),
    (
        "flow-wildcard-before-tested-return",
        "fn entry()->u64{let mut x=7u8;match x{_ if {x=8;return 7}=>99,0=>0,_=>x as u64}}fn main(){check(entry()==7);}",
    ),
    (
        "flow-wildcard-before-tested-break",
        "fn entry()->u64{let mut x=7u8;loop{match x{_ if {x=8;break;}=>(),0=>(),_=>()}}x as u64}fn main(){check(entry()==8);}",
    ),
    (
        "flow-guard-local-move-divergent",
        "struct N(u8);fn entry()->u64{let y=N(7);match 7u8{7|_ if let n=y&&{return n.0 as u64;}=>99,_=>99}}fn main(){check(entry()==7);}",
    ),
    (
        "flow-guard-local-move-irrefutable-true",
        "struct N(u8);fn entry()->u64{let y=N(7);match 7u8{7|_ if let n=y=>n.0 as u64,_=>99}}fn main(){check(entry()==7);}",
    ),
    (
        "flow-enum-payload-move",
        "struct N(u8);enum E{A(N),B}fn entry()->u64{let x=E::A(N(7));match x{E::A(_) if {let E::A(n)=x else{return 98;};n.0==0}=>99,_=>7}}fn main(){check(entry()==7);}",
    ),
    (
        "flow-variant-full-replacement-return",
        "enum E{A(u8),B}fn entry()->u64{let mut x=E::A(7);match x{E::A(_) if {x=E::B;return 7;}=>99,_=>98}}fn main(){check(entry()==7);}",
    ),
    (
        "order-tuple-order",
        "fn entry()->u64{let mut log=0u64;match ((1u8,2u8),(3u8,4u8)){((a,_)|(_,a),(b,_)|(_,b)) if {log=log*100+(a*10+b) as u64;false}=>(),_=>()}log}fn main(){check(entry()==13142324);}",
    ),
    (
        "order-record-order",
        "struct S{a:(u8,u8),b:(u8,u8)}fn entry()->u64{let mut log=0u64;match (S{a:(1,2),b:(3,4)}){S{b:(b,_)|(_,b),a:(a,_)|(_,a)} if {log=log*100+(a*10+b) as u64;false}=>(),_=>()}log}fn main(){check(entry()==13231424);}",
    ),
    (
        "order-nested-order",
        "fn entry()->u64{let mut log=0u64;match (1u8,2u8){((a,_)|(_,a))|((_,a)|(a,_)) if {log=log*10+a as u64;false}=>(),_=>()}log}fn main(){check(entry()==1221);}",
    ),
    (
        "order-rejected-prefixes",
        "fn entry()->u64{let mut log=0u64;match ((1u8,2u8),(3u8,4u8)){((a,9)|(a,_)|(_,a),(9,b)|(b,_)|(_,b)) if {log=log*100+(a*10+b) as u64;false}=>(),_=>()}log}fn main(){check(entry()==13142324);}",
    ),
    (
        "order-parent-skips-inactive-choices",
        "fn entry()->u64{let mut log=0u64;match ((1u8,2u8),(3u8,4u8)){((a,_)|(_,a),_)|(_,(a,_)|(_,a)) if {log=log*10+a as u64;false}=>(),_=>()}log}fn main(){check(entry()==1234);}",
    ),
    (
        "order-success-prefix",
        "fn entry()->u64{let mut log=0u64;let n=match ((1u8,2u8),(3u8,4u8)){((a,_)|(_,a),(b,_)|(_,b)) if {log=log*100+(a*10+b) as u64;a==2&&b==3}=>a*10+b,_=>99};log*100+n as u64}fn main(){check(entry()==13142323);}",
    ),
    (
        "order-record-single-tests",
        "struct S{a:(u8,u8),b:(u8,u8),z:bool}fn entry()->u64{let mut log=0u64;match (S{a:(1,2),b:(3,4),z:true}){S{b:(b,_)|(_,b),z:true,a:(a,_)|(_,a)} if {log=log*100+(a*10+b) as u64;false}=>(),_=>()}log}fn main(){check(entry()==13231424);}",
    ),
    (
        "unused-return",
        "fn main(){let mut x=7u8;match x{n if {x=8;return;}=>(),_=>()};let _=x;}",
    ),
    (
        "nested-guard-views",
        "struct N(u8);fn main(){let x=N(7);let v=match x{n if match n{N(v) if v>0=>true,_=>false}=>n.0,_=>99};check(v==7);}",
    ),
    (
        "dynamic-input-once",
        "fn main(){let x=[(1u8,2u8),(3u8,4u8)];let mut i=0;let mut log=0;match x[{i+=1;0}]{(n,_)|(_,n) if {log=log*10+n;false}=>(),_=>()}check(i==1&&log==12);}",
    ),
    (
        "nested-retries",
        "fn main(){let mut log=0;match (1u8,2u8){(n,_)|(_,n) if {let mut c=0;match n{1|_ if {c+=1;false}=>(),_=>()}log=log*10+c;false}=>(),_=>()}check(log==21);}",
    ),
    (
        "mutable-after-success",
        "fn main(){let x=7u8;let v=match x{mut n if n>0=>{n+=1;n},_=>99};check(v==8&&x==7);}",
    ),
    (
        "failed-view-retains-owner",
        "struct N(u8);fn take(x:N)->u8{x.0}fn main(){let x=N(7);let v=match x{n if n.0==0=>take(n),_=>take(x)};check(v==7);}",
    ),
    (
        "multiple-guards",
        "fn f(x:u8)->u8{match x{n if n==0=>10,n if n==1=>20,n if n==2=>30,n=>n}}fn main(){check(f(0)==10&&f(1)==20&&f(2)==30&&f(7)==7);}",
    ),
    (
        "at-or-retry",
        "fn main(){let mut c=0;let v=match 1u8{n@(1|_) if {c+=1;c==2}=>n,_=>99};check(v==1&&c==2);}",
    ),
    (
        "array-rest-views",
        "fn main(){let x=[1u8,2u8,3u8];let mut c=0;match x{[a,rest@..]|[rest@..,a] if {c+=a+rest[0]+rest[1];false}=>(),_=>()}check(c==12);}",
    ),
    (
        "guard-let-retry-storage",
        "enum E{A(u8),B}fn main(){let mut c=0;let v=match (1u8,2u8){(n,_)|(_,n) if let E::A(m)={c+=1;if n==1{E::B}else{E::A(n+3)}}&&m>0=>m,_=>99};check(c==2&&v==5);}",
    ),
    (
        "guard-continue-storage",
        "fn main(){let mut c=0;let mut sum=0;loop{c+=1;if c==4{break;}match c{n if {if n==2{continue;}true}=>sum+=n,_=>()}}check(sum==4);}",
    ),
    (
        "nested-constructor-retry",
        "enum E{A((u8,u8)),B((u8,u8))}fn f(x:(E,(u8,u8)))->u64{let mut log=0u64;match x{(E::A((a,_))|E::B((_,a)),(b,_)|(_,b)) if {log=log*100+(a*10+b) as u64;false}=>(),_=>()}log}fn main(){check(f((E::A((1,2)),(3,4)))==1314&&f((E::B((1,2)),(3,4)))==2324);}",
    ),
    (
        "zero-size-view",
        "struct N;fn take(_:N)->u8{7}fn main(){let v=match N{n if true=>take(n),_=>99};check(v==7);}",
    ),
    (
        "empty-move-element",
        "struct N;fn main(){let x=[N];match x{[_,n@..] if {let[a]=x;let _=a;true}=>{let _=n;},_=>()}}",
    ),
    (
        "empty-diverging-replacement",
        "fn main(){let mut x:[u8;0]=[];match x{[n@..] if {x=[];return;}=>{let _=n;},_=>()}}",
    ),
    (
        "empty-empty-after-move",
        "struct N(u8);fn main(){let x=[N(1)];let [a]=x;match x{[_,n@..] if true=>{let _=n;},_=>()}let _=a;}",
    ),
    (
        "empty-rest-after-disjoint-move",
        "struct N(u8);fn main(){let x=[N(1),N(2)];let [a,_]=x;match x{[_,n@..] if n[0].0==2=>{let _=n;},_=>()}let _=a;}",
    ),
    (
        "empty-field-after-disjoint-move",
        "struct N(u8);fn main(){let x=[N(1),N(2)];let [a,_]=x;match x{[_,n] if n.0==2=>{let _=n;},_=>()}let _=a;}",
    ),
    (
        "empty-empty-disjoint-struct-write",
        "fn main(){let mut x=([],1u8);match x{(n,_) if {x.1=2;true}=>{let _: [u8;0]=n;},_=>()}}",
    ),
    (
        "empty-empty-rest-disjoint-struct-write",
        "fn main(){let mut x=([],1u8);match x{([n@..],_) if {x.1=2;true}=>{let _: [u8;0]=n;},_=>()}}",
    ),
    (
        "empty-empty-read-after-disjoint-move",
        "struct N(u8);fn main(){let x=[N(1)];match x{[_,n@..] if {let[a]=x;let _=a;let _=n;true}=>(),_=>()}}",
    ),
    (
        "empty-guard-empty-rest-move",
        "struct N;fn main(){let x:[N;0]=[];match x{[n@..] if {let [m@..]=x;let _=m;true}=>{let _=n;},_=>()}}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "contract-bound-write",
        "fn entry()->u64{let mut x=(7u8,1u8);match x{(n,_) if {x.0+=1;n==0}=>99,_=>x.0 as u64}}\nfn main(){}",
    ),
    (
        "contract-tested-write",
        "fn entry()->u64{let mut x=(7u8,0u8);match x{(n,0) if {x.1=1;n==0}=>99,_=>x.1 as u64}}\nfn main(){}",
    ),
    (
        "contract-retry-mutated-test",
        "fn entry()->u64{let mut x=(7u8,0u8);let mut c=0;match x{(n,0)|(n,1) if {x.1=1;c+=1;n==0}=>99,_=>c}}\nfn main(){}",
    ),
    (
        "contract-binding-write",
        "fn entry()->u64{match 7u8{mut n if {n=8;true}=>n as u64,_=>0}}\nfn main(){}",
    ),
    (
        "contract-guard-move",
        "struct N(u8);fn take(_:N)->bool{true}fn entry()->u64{let x=N(7);match x{n if take(n)=>1,_=>2}}\nfn main(){}",
    ),
    (
        "contract-uninitialized-retry",
        "fn entry()->u64{let x:u8;match x{_|0 if false=>1,_=>2}}\nfn main(){}",
    ),
    (
        "places-later-tested-field",
        "fn entry()->u64{let mut x=(7u8,0u8);match x{(n,_) if {x.1=1;n==0}=>99,(_,0)=>0,_=>x.1 as u64}}fn main(){}",
    ),
    (
        "places-earlier-tested-field",
        "fn entry()->u64{let mut x=(7u8,1u8);match x{(_,0)=>0,(n,_) if {x.1=1;n==0}=>99,_=>x.1 as u64}}fn main(){}",
    ),
    (
        "places-untested-array-index",
        "fn entry()->u64{let mut x=[7u8,1u8];match x{[n,_] if {x[1]=2;n==0}=>99,_=>x[1] as u64}}fn main(){}",
    ),
    (
        "places-wildcard-later-test",
        "fn entry()->u64{let mut x=(7u8,1u8);match x{_ if {x=(8,2);false}=>99,(_,0)=>0,_=>x.1 as u64}}fn main(){}",
    ),
    (
        "places-guard-local-retry-move",
        "struct N(u8);fn entry()->u64{let y=N(7);match 7u8{7|_ if let n=y&&true=>n.0 as u64,_=>99}}fn main(){}",
    ),
    (
        "places-guard-from-protected",
        "struct N(u8);struct S(N);fn entry()->u64{let y=S(N(7));match y{x if let S(n)=x=>n.0 as u64,_=>99}}fn main(){}",
    ),
    (
        "places-guard-let-prefix-failure",
        "struct N(u8);fn entry()->u64{let y=N(7);match 7u8{_ if let n=y&&false=>n.0 as u64,_=>y.0 as u64}}fn main(){}",
    ),
    (
        "flow-uninitialized-binding-return",
        "fn entry()->u64{let x:u8;match x{n if return 7=>n as u64,_=>99}}fn main(){}",
    ),
    (
        "flow-uninitialized-or-return",
        "fn entry()->u64{let x:u8;match x{_|0 if return 7=>99,_=>0}}fn main(){}",
    ),
    (
        "flow-binding-inactive-alternative",
        "fn entry()->u64{let x:(u8,u8);match x{(n,_)|(_,n) if return 7=>n as u64,_=>99}}fn main(){}",
    ),
    (
        "flow-guard-initializes-fallback",
        "fn entry()->u64{let x:u8;match 0{_ if {x=7;false}=>99,_=>x as u64}}fn main(){}",
    ),
    (
        "unused-false",
        "fn main(){let mut x=7u8;match x{n if {x=8;false}=>(),_=>()};let _=x;}",
    ),
    (
        "unused-true",
        "fn main(){let mut x=7u8;match x{n if {x=8;true}=>(),_=>()};let _=x;}",
    ),
    (
        "disjoint-repeated-outer-move",
        "struct N;fn main(){let y=N;match 1u8{1|2 if let n=y&&true=>{let _=n;},_=>()}}",
    ),
    (
        "packed-field-view",
        "#[repr(packed)]struct P{a:u8,b:u64}fn main(){let p=P{a:0,b:7};match p{P{b:n,..} if n==7=>(),_=>()}}",
    ),
    (
        "packed-zero-view",
        "#[repr(packed)]struct P{a:u8,z:[u64;0]}fn main(){let p=P{a:0,z:[]};match p{P{z:n,..} if true=>{let _=n;},_=>()}}",
    ),
    (
        "nested-view-move",
        "struct N(u8);fn take(_:N)->bool{true}fn main(){let x=N(7);match x{n if match n{m if true=>take(m),_=>false}=>(),_=>()}}",
    ),
    (
        "empty-replace-zero",
        "fn main(){let mut x:[u8;0]=[];match x{[n@..] if {x=[];true}=>{let _=n;},_=>()}}",
    ),
    (
        "empty-replace-parent",
        "fn main(){let mut x=[7u8];match x{[_,n@..] if {x=[8];true}=>{let _=n;},_=>()}}",
    ),
    (
        "empty-uninitialized-zero",
        "fn main(){let x:[u8;0];match x{[n@..] if true=>{let _=n;},_=>()}}",
    ),
    (
        "empty-uninitialized-parent",
        "struct N;fn main(){let x:[N;1];match x{[_,n@..] if true=>{let _=n;},_=>()}}",
    ),
    (
        "empty-dynamic-index",
        "fn main(){let mut x=[7u8];match x{[_,n@..] if {x[0]=8;true}=>{let _=n;},_=>()}}",
    ),
    (
        "empty-empty-guard-value",
        "fn main(){let mut x:[u8;0]=[];match x{[n@..] if {let _=n;x=[];true}=>(),_=>()}}",
    ),
    (
        "empty-empty-view-subpattern",
        "fn main(){let mut x:[u8;0]=[];match x{[n@..] if {match n{[m@..] if {x=[];true}=>{let _=m;},_=>()};true}=>(),_=>()}}",
    ),
    (
        "empty-guard-whole-empty-move",
        "struct N;fn take(_: [N;0])->bool{true}fn main(){let x=[];match x{[n@..] if take(x)=>{let _: [N;0]=n;},_=>()}}",
    ),
];
fn folder(name: &str) -> Result<PathBuf, std::io::Error> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work")
        .join(name)
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
#[test]
fn guard_programs_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let dir = folder("guard-execution")?;
    for (name, source) in ACCEPTED {
        let file = dir.join(format!("{name}.rs"));
        std::fs::write(&file, format!("{CHECK}{source}"))?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        std::fs::write(file.with_extension("lowir"), &result.stdout)?;
        std::fs::write(file.with_extension("candidate.stderr"), &result.stderr)?;
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
        let exe = dir.join(name);
        let independent = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "guard_values",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
            ])
            .arg(&file)
            .arg("-o")
            .arg(&exe)
            .output()?;
        std::fs::write(file.with_extension("rustc.stderr"), &independent.stderr)?;
        assert!(
            independent.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let ran = Command::new(exe).output()?;
        assert!(
            ran.status.success() && ran.stdout.is_empty() && ran.stderr.is_empty(),
            "{name}: {ran:?}"
        );
        let again = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&file)
            .output()?;
        assert!(again.status.success() && again.stderr.is_empty());
        assert_eq!(result.stdout, again.stdout);
        let count = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--count-lowir")
            .arg(&file)
            .output()?;
        assert!(count.status.success() && count.stderr.is_empty());
    }
    Ok(())
}
#[test]
fn guard_rejections_agree_with_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let dir = folder("guard-errors")?;
    for (name, source) in REJECTED {
        let file = dir.join(format!("{name}.rs"));
        std::fs::write(&file, format!("{CHECK}{source}"))?;
        for mode in ["--emit-lowir", "--count-lowir"] {
            let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&file)
                .output()?;
            std::fs::write(file.with_extension("candidate.stderr"), &candidate.stderr)?;
            assert_eq!(candidate.status.code(), Some(1), "{name}");
            assert!(candidate.stdout.is_empty() && !candidate.stderr.is_empty());
        }
        let independent = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "guard_rejections",
                "--emit=metadata",
            ])
            .arg(&file)
            .arg("-o")
            .arg(dir.join("metadata"))
            .output()?;
        std::fs::write(file.with_extension("rustc.stderr"), &independent.stderr)?;
        assert_eq!(independent.status.code(), Some(1), "{name}");
    }
    Ok(())
}

#[test]
fn guard_retry_graphs_do_not_expand_products() -> Result<(), Box<dyn std::error::Error>> {
    let dir = folder("guard-scaling")?;
    for width in [4, 16, 48] {
        let wide_input = vec!["0u8"; width].join(",");
        let wide_pattern = vec!["0|_"; width].join(",");
        let mut deep_input = "0u8".to_owned();
        let mut deep_pattern = "0|_".to_owned();
        for _ in 0..width {
            deep_input = format!("({deep_input},0u8)");
            deep_pattern = format!("({deep_pattern},0|_)");
        }
        for (name, input, pattern, condition, expected) in [
            (
                "wide",
                format!("({wide_input},0u8)"),
                format!("({wide_pattern},1)"),
                "false",
                0,
            ),
            ("deep", deep_input, deep_pattern, "true", 1),
        ] {
            let file = dir.join(format!("{name}-{width}.rs"));
            std::fs::write(
                &file,
                format!(
                    "{CHECK}fn main(){{let mut calls=0;match {input}{{{pattern} if {{calls+=1;{condition}}}=>(),_=>()}}check(calls=={expected});}}"
                ),
            )?;
            let body = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--count-body-ir")
                .arg(&file)
                .output()?;
            assert!(
                body.status.success(),
                "{name}/{width}: {}",
                String::from_utf8_lossy(&body.stderr)
            );
            let text = std::str::from_utf8(&body.stdout)?;
            for (key, limit) in [("blocks=", 40), ("statements=", 70)] {
                let count: usize = text
                    .split(key)
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .ok_or("missing count")?
                    .parse()?;
                assert!(count < limit * width + 100, "{name}/{width}/{key}: {count}");
            }
            let lowir = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--emit-lowir")
                .arg(&file)
                .output()?;
            assert!(
                lowir.status.success(),
                "{name}/{width}: {}",
                String::from_utf8_lossy(&lowir.stderr)
            );
            assert!(lowir.stdout.len() < 12_000 * width);
            std::fs::write(file.with_extension("lowir"), lowir.stdout)?;
        }
    }
    Ok(())
}
