#[path = "helpers/destructor_cases.rs"]
mod destructor_cases;
use rustgm::{
    body,
    borrow::{
        destruction::{self, glue::Catalog},
        loans,
        ownership::initialization,
        regions,
    },
    types::{destruction::regions::Regions, outlives::LiveRegions},
};
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

fn check(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build(&report, &facts)?;
    body::promote::analyze(&report, &mut facts, &mut program)?;
    let contracts = body::regions::collect(&facts)?;
    let proof = regions::prepare_destruction(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )?;
    let methods = &program.drops.as_ref().ok_or("methods")?.methods;
    let mut catalog = Catalog::new(&facts.types, &facts.aggregates, &facts.members, methods)?;
    let mut drops = Regions::new(methods.iter().map(|m| m.item))?;
    let mut live = LiveRegions::default();
    for cfg in &program.bodies {
        let contract = proof
            .bodies
            .iter()
            .find(|p| p.source == cfg.source)
            .ok_or("contract")?;
        let init = initialization::analyze(
            &facts.types,
            &facts.aggregates,
            &facts.derives,
            &mut facts.copies,
            cfg,
        )
        .map_err(|e| e.message)?;
        let plan = destruction::select(&mut catalog, &init).map_err(|e| e.message)?;
        loans::check_destruction(
            &facts.types,
            &facts.aggregates,
            &mut live,
            &mut drops,
            &init,
            contract,
            &plan,
        )?;
    }
    Ok(())
}
const D: &str = "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn observe(_:u8){}";
#[test]
fn destructor_lifetimes_reject_early_access_and_distinguish_whole_from_field_moves()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            "scope-early",
            "fn f(){let mut a=0;let d=D(&mut a);observe(a);}",
            false,
        ),
        (
            "scope-ended",
            "fn f(){let mut a=0;{let d=D(&mut a);}observe(a);}",
            true,
        ),
        (
            "whole-move",
            "fn f(){let mut a=0;let mut b=0;let p=(D(&mut a),D(&mut b));{let _all=p;}observe(a);observe(b);}",
            true,
        ),
        (
            "field-moves",
            "fn f(){let mut a=0;let mut b=0;let p=(D(&mut a),D(&mut b));{let (_a,_b)=p;}observe(a);observe(b);}",
            false,
        ),
        (
            "partial-move",
            "fn f(){let mut a=0;let mut b=0;let p=(D(&mut a),D(&mut b));{let _a=p.0;}observe(a);}",
            false,
        ),
        (
            "plain-wrapper-field",
            "struct W<'a,'b>(D<'a>,&'b mut u8);fn f(){let mut a=0;let mut b=0;let w=W(D(&mut a),&mut b);observe(b);}",
            true,
        ),
        (
            "custom-wrapper-field",
            "struct W<'a,'b>(D<'a>,&'b mut u8);impl Drop for W<'_,'_>{fn drop(&mut self){}}fn f(){let mut a=0;let mut b=0;let w=W(D(&mut a),&mut b);observe(b);}",
            false,
        ),
        (
            "declaration-order",
            "fn f(){let d;let mut a=0;d=D(&mut a);}",
            false,
        ),
        (
            "ordinary-borrow",
            "fn f(){let mut a=0;{let d=D(&mut a);}let r=&a;a=2;observe(*r);}",
            false,
        ),
        (
            "replacement",
            "fn f(){let mut a=0;let mut b=0;let mut d=D(&mut a);d=D(&mut b);}",
            true,
        ),
        (
            "loop",
            "fn f(mut n:u8){let mut a=0;while n>0{{let d=D(&mut a);}observe(a);n-=1;}}",
            true,
        ),
        (
            "self-borrow",
            "enum E<'a>{None,Full(&'a u8)}impl Drop for E<'_>{fn drop(&mut self){}}fn f(){let mut p=(0u8,E::None);p.1=E::Full(&p.0);}",
            false,
        ),
    ];
    for (name, source, accepted) in cases {
        let source = format!("{D}{source}");
        rustc::qualify(&format!("destructor-loan-{name}"), &source, accepted)?;
        let result = check(&source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
#[test]
fn extended_temporary_storage_ends_after_the_destructor_that_uses_it()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = "struct D<'a>(&'a u8);impl Drop for D<'_>{fn drop(&mut self){observe(*self.0);}}fn observe(_:u8){}fn make()->u8{3}";
    let cases = [
        (
            "tuple-extended",
            "fn f(){let d=(&make(),D(&make()));}",
            true,
        ),
        ("bound-reference", "fn f(){let r=&make();let d=D(r);}", true),
        ("record-extended", "fn f(){let d=D{0:&make()};}", true),
        ("tuple-constructor", "fn f(){let d=D(&make());}", true),
        (
            "ordinary-call",
            "fn wrap(x:&u8)->D<'_>{D(x)}fn f(){let d=wrap(&make());}",
            false,
        ),
        (
            "inner-scope",
            "fn f(){let d;{let x=make();d=D(&x);}}",
            false,
        ),
        (
            "nested-return",
            "fn f(){let r=&make();{let d=D(r);return;}}",
            true,
        ),
    ];
    for (name, source, accepted) in cases {
        let source = format!("{prefix}{source}");
        rustc::qualify(&format!("destructor-storage-{name}"), &source, accepted)?;
        let result = check(&source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
fn counter_programs_qualify_the_destructor_loan_proof() -> Result<(), Box<dyn std::error::Error>> {
    let cases = destructor_cases::CASES;
    for &(name, path, accepted) in cases {
        let source = destructor_cases::source(path)?;
        rustc::qualify(&format!("destructor-counter-{name}"), &source, accepted)?;
        let result = check(&source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
fn replacement_ends_old_loans_only_when_every_live_holder_is_gone()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            "direct",
            "fn f(){let mut a=0;let mut b=0;let mut d=D(&mut a);d=D(&mut b);observe(a);}",
            true,
        ),
        (
            "new-loan",
            "fn f(){let mut a=0;let mut b=0;let mut d=D(&mut a);d=D(&mut b);observe(b);}",
            false,
        ),
        (
            "moved-old",
            "fn f(){let mut a=0;let mut b=0;let mut d=D(&mut a);let old=d;d=D(&mut b);observe(a);}",
            false,
        ),
        (
            "moved-ended",
            "fn f(){let mut a=0;let mut b=0;let mut d=D(&mut a);{let old=d;}d=D(&mut b);observe(a);}",
            true,
        ),
        (
            "borrowed-slot",
            "fn f(){let mut a=0;let mut b=0;let mut d=D(&mut a);let alias=&d;d=D(&mut b);observe(*alias.0);}",
            false,
        ),
        (
            "copied-alias",
            "fn f(){let mut a=0;let b=0;let r=&a;let alias=r;let mut d=S(r);d=S(&b);a=1;observe(*alias);}",
            false,
        ),
        (
            "ended-alias",
            "fn f(){let mut a=0;let b=0;let r=&a;let alias=r;let mut d=S(r);observe(*alias);d=S(&b);a=1;observe(a);}",
            true,
        ),
        (
            "same-referent",
            "fn f(){let mut a=0;let mut d=S(&a);d=S(&a);a=1;}",
            false,
        ),
        (
            "one-branch",
            "fn f(flag:bool){let mut a=0;let mut b=0;let mut d=D(&mut a);if flag{d=D(&mut b);}observe(a);}",
            false,
        ),
        (
            "both-branches",
            "fn f(flag:bool){let mut a=0;let mut b=0;let mut c=0;let mut d=D(&mut a);if flag{d=D(&mut b);}else{d=D(&mut c);}observe(a);}",
            true,
        ),
        (
            "breaking-loop",
            "fn f(){let mut a=0;let mut b=0;let mut d=D(&mut a);loop{d=D(&mut b);break;}observe(a);}",
            true,
        ),
        (
            "rhs-reborrow",
            "fn f(){let mut a=0;let mut d=D(&mut a);d=D(d.0);}",
            false,
        ),
        (
            "indirect-conservative",
            "fn replace<'a>(slot:&mut D<'a>,value:D<'a>){*slot=value;}fn f(){let mut a=0;let mut b=0;let mut d=D(&mut a);replace(&mut d,D(&mut b));observe(a);}",
            false,
        ),
        (
            "projected-conservative",
            "fn f(){let mut a=0;let mut b=0;let mut c=0;let mut p=(D(&mut a),D(&mut c));p.0=D(&mut b);observe(a);}",
            false,
        ),
        (
            "whole-product",
            "fn f(){let mut a=0;let mut b=0;let mut p=(D(&mut a),);p=(D(&mut b),);observe(a);}",
            true,
        ),
    ];
    for (name, source, accepted) in cases {
        let source = format!(
            "{D}struct S<'a>(&'a u8);impl Drop for S<'_>{{fn drop(&mut self){{observe(*self.0);}}}}{source}"
        );
        rustc::qualify(&format!("destructor-replace-{name}"), &source, accepted)?;
        let result = check(&source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
