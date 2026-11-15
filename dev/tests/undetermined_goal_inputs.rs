//! A trait goal's undetermined projected input is published before a candidate is chosen.
use crate::{checked_sources as checked, native_references as native};
use std::path::PathBuf;

/// A one-impl family: `<Range<?A> as Iter>::Item` is `?A` whatever `?A` becomes.
const FAMILY: &str = "struct Range<A>(A,A);trait Step{}impl Step for u32{}impl Step for u64{}impl Step for i32{}impl<T>Step for Opt<T>{}enum Opt<T>{No,Yes(T)}\ntrait Iter:Sized{type Item;fn first(self)->Self::Item;fn sum_into<S:Sum<Self::Item>>(self)->S{S::sum_of(self)}}\nimpl<A:Step>Iter for Range<A>{type Item=A;fn first(self)->A{self.0}}\ntrait Sum<A=Self>:Sized{fn sum_of<I:Iter<Item=A>>(iter:I)->Self;}\n";
const OWNED: &str =
    "impl Sum<u32>for u32{fn sum_of<I:Iter<Item=u32>>(iter:I)->u32{iter.first()+100}}\n";
const BORROWED: &str = "impl<'a>Sum<&'a u32>for u32{fn sum_of<I:Iter<Item=&'a u32>>(iter:I)->u32{*iter.first()+200}}\n";
const SIGNED: &str = "impl Sum<i32>for u32{fn sum_of<I:Iter<Item=i32>>(iter:I)->u32{if iter.first()==1{300}else{0}}}\n";
const WIDE: &str = "impl Sum<u64>for u32{fn sum_of<I:Iter<Item=u64>>(iter:I)->u32{if iter.first()==1{400}else{0}}}\n";
const OPTIONAL: &str = "impl Sum<Opt<u32>>for u32{fn sum_of<I:Iter<Item=Opt<u32>>>(iter:I)->u32{match iter.first(){Opt::Yes(v)=>v,Opt::No=>500}}}\n";
const OPTIONAL_WIDE: &str = "impl Sum<Opt<u64>>for u32{fn sum_of<I:Iter<Item=Opt<u64>>>(iter:I)->u32{match iter.first(){Opt::Yes(_)=>0,Opt::No=>600}}}\n";

/// rustc 1.98.0 accepts each program, and each executes its checks.
const ACCEPTED: &[(&str, &[&str], &str)] = &[
    // Already accepted: the hole is the trait argument itself.
    (
        "direct-hole",
        &[OWNED, BORROWED],
        "fn total<A:Step,S:Sum<A>>(r:Range<A>)->S{S::sum_of(r)}\nfn main(){check(total::<_,u32>(Range(1,10))==101);}\n",
    ),
    (
        "method",
        &[OWNED, BORROWED],
        "fn main(){let s=Range(1,10).sum_into::<u32>();check(s==101);}\n",
    ),
    (
        "annotated-result",
        &[OWNED, BORROWED],
        "fn main(){let s:u32=Range(1,10).sum_into();check(s==101);}\n",
    ),
    (
        "function",
        &[OWNED, BORROWED],
        "fn total<I:Iter,S:Sum<I::Item>>(i:I)->S{S::sum_of(i)}\nfn main(){check(total::<_,u32>(Range(1,10))==101);}\n",
    ),
    (
        "projected-subject",
        &[OWNED],
        "trait Show{fn show(self)->u32;}impl Show for u32{fn show(self)->u32{self+7}}fn shown<I:Iter>(i:I)->u32 where I::Item:Show{i.first().show()}\nfn main(){check(shown(Range(1,10))==8);check(Range(1,2).sum_into::<u32>()==101);}\n",
    ),
    (
        "adapter",
        &[OWNED, BORROWED],
        "struct Rev<I>(I);impl<I:Iter>Iter for Rev<I>{type Item=I::Item;fn first(self)->I::Item{self.0.first()}}\nfn main(){check(Rev(Rev(Range(1,10))).sum_into::<u32>()==101);}\n",
    ),
    (
        "general-hole",
        &[OWNED, OPTIONAL],
        "fn main(){check(Range(Opt::No,Opt::No).sum_into::<u32>()==500);check(Range(1,2).sum_into::<u32>()==101);}\n",
    ),
    // The projection also sits beneath a function pointer's binder.
    (
        "callable-input",
        &[],
        "trait Pair<A>:Sized{fn pair(a:A)->Self;}impl<F>Pair<(u32,F)>for u32{fn pair(a:(u32,F))->u32{a.0+100}}impl<'a,F>Pair<(&'a u32,F)>for u32{fn pair(a:(&'a u32,F))->u32{*a.0+200}}\nfn pick<I:Iter,S:Pair<(I::Item,fn(I::Item)->I::Item)>>(i:I,f:fn(I::Item)->I::Item)->S{S::pair((i.first(),f))}fn same<T>(x:T)->T{x}\nfn main(){let s:u32=pick(Range(1,2),same);check(s==101);}\n",
    ),
    (
        "closure-subject",
        &[],
        "trait Each<A>:Sized{fn each(self,a:A)->u32;}impl<F:FnMut(u32)->u32>Each<u32>for F{fn each(mut self,a:u32)->u32{self(a)+100}}impl<'a,F:FnMut(&'a u32)->u32>Each<&'a u32>for F{fn each(mut self,a:&'a u32)->u32{self(a)+200}}\nfn each<I:Iter,F:Each<I::Item>>(i:I,f:F)->u32{f.each(i.first())}\nfn main(){check(each(Range(1,2),|x|x+1)==102);}\n",
    ),
    // Already accepted: the deferred goal's callable input holds only a
    // determined projection, which is not registered again at each retry.
    (
        "determined-callable-input",
        &[],
        "trait Pair<A>:Sized{fn pair(a:A)->Self;}impl<F>Pair<(i32,F)>for u32{fn pair(a:(i32,F))->u32{if a.0==1{300}else{0}}}impl<F>Pair<(u32,F)>for u32{fn pair(a:(u32,F))->u32{a.0+100}}\nfn pick<I:Iter,X,S:Pair<(X,fn(I::Item)->I::Item)>>(_i:I,x:X,f:fn(I::Item)->I::Item)->S{S::pair((x,f))}\nfn run<I:Iter>(i:I,f:fn(I::Item)->I::Item)->u32{let x=1;let s:u32=pick(i,x,f);let y:u32=x;s+y}fn same(x:u32)->u32{x}\nfn main(){check(run(Range(1u32,2),same)==102);}\n",
    ),
    // Two integer candidates stay ambiguous until the literal falls back to `i32`.
    (
        "integer-fallback",
        &[OWNED, SIGNED],
        "fn main(){check(Range(1,10).sum_into::<u32>()==300);check(Range(1u32,10).sum_into::<u32>()==101);}\n",
    ),
];

const AMBIGUOUS: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
/// rustc 1.98.0 rejects each program; the frontend rejects it for the stated reason.
const REJECTED: &[(&str, &[&str], &str, &str)] = &[
    (
        "ambiguous-integers",
        &[OWNED, WIDE],
        "fn main(){check(Range(1,10).sum_into::<u32>()==101);}\n",
        AMBIGUOUS,
    ),
    (
        "borrowed-only",
        &[BORROWED],
        "fn main(){check(Range(1,10).sum_into::<u32>()==101);}\n",
        AMBIGUOUS,
    ),
    (
        "determined-mismatch",
        &[OWNED, BORROWED],
        "fn main(){check(Range(1u64,10).sum_into::<u32>()==101);}\n",
        AMBIGUOUS,
    ),
    (
        "ambiguous-general-hole",
        &[OPTIONAL, OPTIONAL_WIDE],
        "fn main(){check(Range(Opt::No,Opt::No).sum_into::<u32>()==500);}\n",
        AMBIGUOUS,
    ),
    // rustc selects at the call and reports the later `u64` (E0308); the
    // frontend selects at its frontier, where the literal is already `u64`.
    (
        "later-conflict",
        &[OWNED, BORROWED],
        "fn main(){let a=1;let s=Range(a,10).sum_into::<u32>();let b:u64=a;check(s==101&&b==1);}\n",
        AMBIGUOUS,
    ),
];

fn program(items: &[&str], body: &str) -> String {
    format!("{FAMILY}{}{body}", items.concat())
}

#[test]
fn undetermined_goal_inputs_select_candidates_as_rustc_does()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/projections/undetermined-goal-inputs/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, items, body) in ACCEPTED {
        let path = evidence.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{}{}", native::CHECK, program(items, body)))?;
        checked::check(&path, &evidence, name, (0, 0, ""))?;
    }
    for &(name, items, body, reason) in REJECTED {
        let path = evidence.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{}{}", native::CHECK, program(items, body)))?;
        checked::check(&path, &evidence, name, (1, 1, reason))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn undetermined_goal_inputs_execute_selected_candidates() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, items, body) in ACCEPTED {
        native::check_cases(
            "parallel/projections/undetermined-goal-inputs/native",
            &[(name, &program(items, body), false)],
        )?;
    }
    Ok(())
}
