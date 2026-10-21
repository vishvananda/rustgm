//! Hidden lifetime closure consumes each demanded definition's actual CFG and borrow proof.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[expect(
    dead_code,
    reason = "this suite uses resolution but routes body inference through the defining-proof pipeline"
)]
#[path = "support/semantic.rs"]
mod semantic;
type ResultTest<T = ()> = Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, &str, bool)] = &[
    (
        "static-marker",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}unsafe impl<'a:'static> Send for W<'a>{}fn make<'a>(x:&'a u8)->impl A+'a{W(x)}static N:u8=7;fn main(){let x=make(&N);need(&x);check(x.f()==7);}",
        true,
    ),
    (
        "local-structural",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a>(x:&'a u8)->impl A+'a{W(x)}fn main(){let n=9u8;let x=make(&n);need(&x);check(x.f()==9);}",
        true,
    ),
    (
        "local-marker-unpaid",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}unsafe impl<'a:'static> Send for W<'a>{}fn make<'a>(x:&'a u8)->impl A+'a{W(x)}fn main(){let n=9u8;let x=make(&n);need(&x);}",
        false,
    ),
    (
        "generic-structural",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a>(x:&'a u8)->impl A+'a{W(x)}fn caller<'a>(x:&'a u8){let x=make(x);need(&x);check(x.f()==11);}fn main(){caller(&11);}",
        true,
    ),
    (
        "generic-marker-unpaid",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}unsafe impl<'a:'static> Send for W<'a>{}fn make<'a>(x:&'a u8)->impl A+'a{W(x)}fn caller<'a>(x:&'a u8){let x=make(x);need(&x);}fn main(){}",
        false,
    ),
    (
        "defining-bound-unpaid",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a>(x:&'a u8)->impl A+'static{W(x)}fn main(){let x=make(&7);need(&x);}",
        false,
    ),
    (
        "local-escape",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make()->impl A{let n=7u8;W(&n)}fn main(){let x=make();need(&x);}",
        false,
    ),
    (
        "pattern-complete",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}const C:(u8,u8)=(1,2);fn make<'a>(x:&'a u8)->impl A+'a{match (1u8,2u8){C=>W(x),_=>W(x)}}fn main(){let n=13u8;let x=make(&n);need(&x);check(x.f()==13);}",
        true,
    ),
    (
        "pattern-incomplete",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}const C:(u8,u8)=(1,2);fn make<'a>(x:&'a u8)->impl A+'a{match (1u8,2u8){C=>W(x)}}fn main(){let n=13u8;let x=make(&n);need(&x);}",
        false,
    ),
    (
        "conditional-captures",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a,'b>(x:&'a u8,y:&'b u8,b:bool)->impl A+'b where 'a:'b{W(if b{x}else{y})}fn main(){let n=17u8;let x=make(&n,&n,true);need(&x);check(x.f()==17);}",
        true,
    ),
    (
        "multiple-results",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a>(x:&'a u8)->(impl A+'a,impl A+'a){(W(x),W(x))}fn main(){let n=19u8;let (x,y)=make(&n);need(&x);need(&y);check(x.f()+y.f()==38);}",
        true,
    ),
    (
        "disjoint-captures",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a,'b>(x:&'a u8,y:&'b u8)->(impl A+use<'a>,impl A+use<'b>){(W(x),W(y))}fn main(){let n=19u8;let m=23u8;let (x,y)=make(&n,&m);need(&y);need(&x);check(x.f()+y.f()==42);}",
        true,
    ),
    (
        "disjoint-static-marker",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}unsafe impl<'a:'static> Send for W<'a>{}fn need<T:Send>(_:&T){}fn make<'a,'b>(x:&'a u8,y:&'b u8)->(impl A+use<'a>,impl A+use<'b>){(W(x),W(y))}static N:u8=19;fn main(){let m=23u8;let (x,y)=make(&N,&m);need(&x);check(x.f()+y.f()==42);}",
        true,
    ),
    (
        "disjoint-marker-unpaid",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}unsafe impl<'a:'static> Send for W<'a>{}fn need<T:Send>(_:&T){}fn make<'a,'b>(x:&'a u8,y:&'b u8)->(impl A+use<'a>,impl A+use<'b>){(W(x),W(y))}static N:u8=19;fn main(){let m=23u8;let (x,y)=make(&N,&m);need(&x);need(&y);}",
        false,
    ),
    (
        "disjoint-marker-unpaid-reversed",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}unsafe impl<'a:'static> Send for W<'a>{}fn need<T:Send>(_:&T){}fn make<'a,'b>(x:&'a u8,y:&'b u8)->(impl A+use<'a>,impl A+use<'b>){(W(x),W(y))}static N:u8=19;fn main(){let m=23u8;let (x,y)=make(&N,&m);need(&y);need(&x);}",
        false,
    ),
    (
        "reborrow",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a>(x:&'a u8)->impl A+'a{let y=&*x;W(y)}fn main(){let n=23u8;let x=make(&n);need(&x);check(x.f()==23);}",
        true,
    ),
    (
        "promoted-result",
        "trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make()->impl A{W(&29u8)}fn main(){let x=make();need(&x);check(x.f()==29);}",
        true,
    ),
];
fn build(source: &str) -> ResultTest<(rustgm::lowir::program::Program, rustgm::pipeline::Work)> {
    build_observed(source, |_| {})
}
fn build_observed(
    source: &str,
    observe: impl FnOnce(&rustgm::sema::model::Declarations),
) -> ResultTest<(rustgm::lowir::program::Program, rustgm::pipeline::Work)> {
    let mut report = semantic::resolve(source)?;
    let facts = rustgm::sema::declarations::analyze(&report)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze(&report, facts)?;
    let work = prepared.work();
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let program = rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    observe(&facts);
    Ok((program, work))
}
#[test]
fn defining_lifetimes_match_independent_language_outcomes() -> ResultTest {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("defining-regions-{name}"), &source, accepted)?;
        let result = build(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.as_ref().err()));
        }
        if let Ok((_, work)) = result {
            assert_eq!(work.templates, 1, "{name}");
            assert!(work.lifetime_facts > 0, "{name}");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn defining_lifetimes_execute() -> ResultTest {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("pa24-pa25-work/native-defining-regions", &cases)
}
#[test]
fn growing_dependencies_build_each_cfg_once_and_consume_appended_facts_once() -> ResultTest {
    use std::fmt::Write;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = format!(
            "{}trait A{{fn f(&self)->u8;}}struct W<'a>(&'a u8);impl A for W<'_>{{fn f(&self)->u8{{*self.0}}}}fn need<T:Send>(_:&T){{}}fn main(){{let x=make0(&7);need(&x);check(x.f()==7);}}",
            native::CHECK
        );
        for index in 0..width {
            write!(source, "fn make{index}<'a>(x:&'a u8)->impl A+'a{{")?;
            if index + 1 < width {
                write!(source, "need(&make{}(x));", index + 1)?;
            }
            source.push_str("W(x)}");
        }
        rustc::qualify(&format!("defining-region-growth-{width}"), &source, true)?;
        let (_, work) = build(&source)?;
        println!("defining bodies={width}: {work:?}");
        assert_eq!(work.templates, width);
        assert!(work.lifetime_facts < width * 100 + 200);
        assert!(work.constant_preparation > 0);
        assert!(work.constant_preparation < width * 20 + 100);
        if let Some(previous) = previous {
            let previous: rustgm::pipeline::Work = previous;
            assert!(work.lifetime_facts < previous.lifetime_facts * 5);
            assert!(work.constant_preparation < previous.constant_preparation * 5);
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
fn unrelated_constants_are_indexed_once_across_defining_bodies() -> ResultTest {
    use std::fmt::Write;
    let mut previous = None;
    for count in [32, 128, 512] {
        let mut source = String::from(
            "trait A{}struct W<'a>(&'a u8);impl A for W<'_>{}fn need<T:Send>(_:&T){}fn main(){let x=make0(&7);need(&x);}",
        );
        for index in 0..count {
            write!(source, "const C{index}:u8=7;")?;
        }
        for index in 0..32 {
            write!(source, "fn make{index}<'a>(x:&'a u8)->impl A+'a{{")?;
            if index + 1 < 32 {
                write!(source, "need(&make{}(x));", index + 1)?;
            }
            source.push_str("W(x)}");
        }
        rustc::qualify(&format!("defining-constant-growth-{count}"), &source, true)?;
        let (_, work) = build_observed(&source, |facts| {
            let constants = &facts.constant_work;
            println!(
                "initializers={count}: indexed={} lookups={} visits={} validations={}",
                constants.initializer_indexed,
                constants.initializer_lookups.get(),
                constants.initializer_visits,
                constants.initializer_validations,
            );
            assert_eq!(constants.initializer_indexed, count);
            // This lower-level defining-body pipeline deliberately skips the
            // public pipeline's eager ordinary initializer validation. Unused
            // constants are indexed once and never queried by these bodies.
            assert_eq!(constants.initializer_visits, 0);
            assert_eq!(constants.initializer_validations, 0);
            assert_eq!(constants.initializer_lookups.get(), 0);
            assert_eq!(facts.initializer_index_bytes(), 4 * facts.definitions.len());
        })?;
        println!("unrelated constants={count}: {work:?}");
        assert_eq!(work.templates, 32);
        assert!(work.constant_preparation < count * 3 + 400);
        // The declaration index is charged above. Retained promotion/query
        // scratch setup must not revisit it for each defining body or width.
        if let Some(prior_steps) = previous {
            assert_eq!(work.constant_preparation, prior_steps);
        }
        previous = Some(work.constant_preparation);
    }
    Ok(())
}
