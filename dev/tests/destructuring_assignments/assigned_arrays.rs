//! An array assignee is rustc's `{ let [lhs0, lhs1] = rhs; leaf0 = lhs0; leaf1 = lhs1; }`:
//! the right operand is evaluated first, then each element is stored to its
//! leaf from left to right.
use super::Result;
use crate::{checked_sources as checked, native_references as native};
use std::path::PathBuf;

/// Each body follows the shared `check` prefix; rustc 1.98.0 accepts and runs it.
const ACCEPTED: &[(&str, &str)] = &[
    (
        "swap",
        "fn main(){let (mut x,mut y)=(1u8,2u8);[x,y]=[y*2,x*2];check(x==4&&y==2);}\n",
    ),
    (
        "nested",
        "fn main(){let (mut a,mut b,mut c,mut d)=(1u8,2u8,3u8,4u8);[(a,b),_]=[(b,a),(c,d)];check(a==2&&b==1);(c,[d,_])=(d,[c,a]);check(c==4&&d==3);[[a,b],[c,d]]=[[d,c],[b,a]];check(a==3&&b==4&&c==1&&d==2);}\n",
    ),
    (
        "moves",
        "struct Token(u8);\nfn rotate<T>(mut a:T,mut b:T)->(T,T){[a,b]=[b,a];(a,b)}\nfn main(){let mut a=Token(1);let mut b=Token(2);[a,b]=[b,a];check(a.0==2&&b.0==1);let c;[c,_]=[a,b];check(c.0==2);let (s,t)=rotate(Token(3),Token(4));check(s.0==4&&t.0==3);}\n",
    ),
    (
        "drops",
        "struct D(*mut u8,u8);\nimpl Drop for D{fn drop(&mut self){unsafe{*self.0+=self.1}}}\nfn main(){let mut n=0u8;let p=&raw mut n;{let mut a=D(p,1);let mut b=D(p,2);check(a.1==1&&b.1==2);[a,b]=[D(p,4),D(p,8)];check(unsafe{*p}==3&&a.1==4);[a,_]=[b,D(p,16)];check(unsafe{*p}==23&&a.1==8);}check(n==31);}\n",
    ),
    (
        "places",
        "struct S{a:u8,b:u8}\nfn main(){let mut s=S{a:1,b:2};let mut v=[3u8,4,5];let mut n=6u8;let r=&mut n;[s.a,v[0],*r]=[s.b,s.a,v[2]];check(s.a==2&&v[0]==1&&n==5);[v[1],v[2]]=[v[0],v[1]];check(v[1]==1&&v[2]==4&&s.b==2);}\n",
    ),
    (
        "order",
        "fn main(){let mut i=0usize;let mut v=[7usize,8,9];[i,v[i]]=[i+1,v[i]+5];check(i==1&&v[0]==7&&v[1]==12&&v[2]==9);[v[i],i]=[i+3,i+1];check(v[1]==4&&i==2);}\n",
    ),
    (
        "captured",
        "fn main(){let (mut x,mut y)=(1u8,2u8);let mut swap=||[x,y]=[y,x];swap();swap();swap();check(x==2&&y==1);}\n",
    ),
];
/// rustc 1.98.0 rejects every control. Typing rejects in all three views;
/// ownership rejects once a body is lowered.
const REJECTED: &[(&str, &str, [i32; 3], &str)] = &[
    (
        "length",
        "fn main(){let (mut x,mut y)=(1u8,2u8);[x,y]=[y,x,3];check(x==2&&y==1);}\n",
        [1, 1, 1],
        "incompatible types",
    ),
    (
        "tuple-source",
        "fn main(){let (mut x,mut y)=(1u8,2u8);[x,y]=(y,x);check(x==2&&y==1);}\n",
        [1, 1, 1],
        "incompatible types",
    ),
    (
        "slice-source",
        "fn first(s:&[u8])->u8{let (mut x,mut y)=(1u8,2u8);check(x+y==3);[x,y]=*s;x+y}\nfn main(){check(first(&[1,2])==3);}\n",
        [1, 1, 1],
        "incompatible types",
    ),
    (
        "compound",
        "fn main(){let (mut x,mut y)=(1u8,2u8);[x,y]+=[y,x];check(x==3&&y==3);}\n",
        [1, 1, 1],
        "assignment requires a place",
    ),
    (
        "immutable-leaf",
        "fn main(){let x=1u8;let mut y=2u8;[x,y]=[y,x];check(x==2&&y==1);}\n",
        [0, 1, 1],
        "cannot assign twice to an immutable local",
    ),
    (
        "moved-source",
        "struct Token(u8);\nfn main(){let p=[Token(1),Token(2)];let (a,b);[a,b]=p;check(a.0==1&&b.0==2&&p[0].0==1);}\n",
        [0, 1, 1],
        "indexed array may be uninitialized or moved",
    ),
    (
        "borrowed-source",
        "struct Token(u8);\nfn take(r:&[Token;2])->u8{let (a,b);[a,b]=*r;a.0+b.0}\nfn main(){check(take(&[Token(1),Token(2)])==3);}\n",
        [0, 1, 1],
        "cannot move a non-Copy value out of a reference",
    ),
];

fn evidence(kind: &str) -> Result<PathBuf> {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/publication/array-assignees/{kind}/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}

#[test]
fn array_assignees_type_and_lower_as_rustc_accepts() -> Result {
    let evidence = evidence("accepted")?;
    for &(name, body) in ACCEPTED {
        let path = evidence.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{}{body}", native::CHECK))?;
        checked::check(&path, &evidence, name, (0, 0, ""))?;
    }
    Ok(())
}

#[test]
fn malformed_array_assignees_are_rejected_as_rustc_rejects() -> Result {
    let evidence = evidence("rejected")?;
    for &(name, body, frontend, reason) in REJECTED {
        let path = evidence.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{}{body}", native::CHECK))?;
        checked::check_modes(&path, &evidence, name, (1, frontend, reason))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn array_assignees_execute_like_rustc() -> Result {
    let cases: Vec<_> = ACCEPTED
        .iter()
        .map(|&(name, body)| (name, body, false))
        .collect();
    native::check_cases("parallel/publication/array-assignees/native", &cases)
}
