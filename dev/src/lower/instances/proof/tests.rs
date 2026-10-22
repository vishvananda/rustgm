//! Exercise the sealed incremental boundary with real semantic and CFG inputs.
#[path = "../../../../tests/invariants/adapter_contracts.rs"]
mod adapters;
use crate::{
    body::{
        self,
        ir::Program,
        regions::{Contract, Contracts},
        templates::Templates,
    },
    borrow::{self, incremental, templates},
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    sema::{self, model::Declarations},
    support::source::SourceFile,
    types::{Kind, instance::Instance, outlives::Queries},
};
use std::path::Path;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn instance_cleanup_shares_completed_types_across_distinct_callees() -> TestResult {
    use std::fmt::Write as _;
    for depth in [4, 16, 64] {
        let mut source = String::from("struct S0<T>(T);");
        for level in 1..=depth {
            write!(source, "struct S{level}<T>(S{}<T>);", level - 1)?;
        }
        for index in 0..depth {
            write!(source, "fn id{index}<T>(x:S{depth}<T>)->S{depth}<T>{{x}}")?;
        }
        source.push_str("fn main(){");
        for index in 0..depth {
            write!(
                source,
                "let f{index}:fn(S{depth}<u8>)->S{depth}<u8>=id{index}::<u8>;"
            )?;
        }
        source.push('}');
        let mut f = fixture(&source)?;
        let checked = originals(&mut f.facts, &f.templates, &f.inputs)?;
        assert_eq!(f.instances.len(), depth);
        let mut cleanup = body::instantiate::Cleanup::new(&f.facts);
        let mut complete = None;
        for instance in f.instances {
            let Some(Kind::FunctionItem { item, .. }) = f.facts.types.kind(instance.ty()) else {
                return Err("missing selected function".into());
            };
            let result = body::instantiate::from_checked_caller(
                &mut f.facts,
                checked.get(item).ok_or("missing template")?,
                instance,
                checked.input(item).ok_or("missing contract")?,
                &mut cleanup,
            )?;
            assert!(!result.destruction.locals.contains(&true));
            assert!(!result.destruction.values.contains(&true));
            assert!(result.body.owned.is_none());
            if let Some(steps) = complete {
                assert_eq!(
                    cleanup.steps(),
                    steps,
                    "another callee reopened shared fields"
                );
            }
            complete = Some(cleanup.steps());
        }
        assert!(cleanup.steps() <= 10 * depth + 64);
    }
    Ok(())
}

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
struct Fixture {
    facts: Declarations,
    templates: Templates,
    inputs: Vec<Contract>,
    instances: Vec<Instance>,
    program: Program,
}
#[expect(
    clippy::default_trait_access,
    reason = "use the resolution entrypoint's defaults without an expansion dependency in lowering"
)]
fn resolve_source(source: &str) -> TestResult<resolve::expansion::Report> {
    Ok(resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Default::default(),
    )
    .and_then(resolve::analyze_expanded)?)
}
fn fixture(source: &str) -> TestResult<Fixture> {
    let report = resolve_source(source)?;
    let mut facts =
        sema::declarations::analyze(&report).and_then(|f| sema::bodies::analyze(&report, f))?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let inputs = body::regions::collect(&facts)?.bodies;
    let mut program = body::build_concrete(&report, &facts)?;
    body::promote::analyze(&report, &mut facts, &mut program)?;
    let mut instances = Vec::new();
    for value in program.bodies.iter().flat_map(|b| &b.values) {
        if let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(value.ty)
            && facts.generics.has_parameters(resolve::model::DefId(item.0))
        {
            let instance = Instance::new(&facts.types, value.ty)?;
            if !instances.contains(&instance) {
                instances.push(instance);
            }
        }
    }
    Ok(Fixture {
        facts,
        templates,
        inputs,
        instances,
        program,
    })
}
fn originals<'a>(
    facts: &mut Declarations,
    templates: &'a Templates,
    inputs: &[Contract],
) -> TestResult<templates::Checked<'a>> {
    let contracts = Contracts {
        bodies: inputs.to_vec(),
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut facts.copies,
    };
    Ok(templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        templates,
        &[],
        |_, _| Err("unexpected field"),
    )?)
}
fn specialize(
    facts: &mut Declarations,
    checked: &templates::Checked<'_>,
    instance: Instance,
) -> TestResult<(body::ir::Body, borrow::regions::Body)> {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
        return Err("missing function instance".into());
    };
    let materialized = body::instantiate::with_contract(
        facts,
        checked.get(item).ok_or("missing original")?,
        instance,
        checked.input(item).ok_or("missing input")?,
    )?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let contract = borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        &materialized.contract,
        &materialized.body,
    )?;
    Ok((materialized.body, contract))
}
fn prove(
    facts: &mut Declarations,
    checked: &templates::Checked<'_>,
    body: body::ir::Body,
    contract: &borrow::regions::Body,
) -> Result<incremental::Proved, borrow::Error> {
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut facts.copies,
    };
    incremental::Verifier::new(None)
        .map_err(|e| borrow::Error {
            node: None,
            message: e.into(),
        })?
        .prove(&mut environment, checked, contract, body, None, None)
}

#[test]
fn concrete_proofs_require_the_matching_instance_owner_and_original() -> TestResult {
    let mut f = fixture("fn id<T>(x:T)->T{x}fn main(){let a=id(7u8);let b=id(8u16);}")?;
    let checked = originals(&mut f.facts, &f.templates, &f.inputs)?;
    assert_eq!(f.instances.len(), 2);
    let first = f.instances[0];
    let second = f.instances[1];
    let (body, mut contract) = specialize(&mut f.facts, &checked, first)?;
    contract.instance = Some(second);
    let error = prove(&mut f.facts, &checked, body, &contract)
        .err()
        .ok_or("swapped proof accepted")?;
    assert!(error.message.contains("another body"));
    let (body, mut contract) = specialize(&mut f.facts, &checked, first)?;
    contract.source.node = crate::syntax::ast::NodeId::new(contract.source.node.get() + 1);
    assert!(prove(&mut f.facts, &checked, body, &contract).is_err());
    let (body, mut contract) = specialize(&mut f.facts, &checked, first)?;
    contract.key = crate::body::ir::BodyKey::Anonymous(body.source);
    assert!(prove(&mut f.facts, &checked, body, &contract).is_err());
    let (mut body, contract) = specialize(&mut f.facts, &checked, first)?;
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = f.facts.types.kind(second.ty())
    else {
        return Err("missing second signature".into());
    };
    body.owner = body::ir::Owner::Function {
        definition: resolve::model::DefId(item.0),
        signature,
    };
    let error = prove(&mut f.facts, &checked, body, &contract)
        .err()
        .ok_or("wrong owner accepted")?;
    assert!(error.message.contains("different owner"));
    let empty = Templates { bodies: vec![] };
    let no_originals = originals(&mut f.facts, &empty, &f.inputs)?;
    let (body, contract) = specialize(&mut f.facts, &checked, first)?;
    let error = prove(&mut f.facts, &no_originals, body, &contract)
        .err()
        .ok_or("unproved original accepted")?;
    assert!(error.message.contains("no checked original"));
    let (body, contract) = specialize(&mut f.facts, &checked, first)?;
    let proved = prove(&mut f.facts, &checked, body, &contract)?;
    // The successful token owns exactly the immutable body that passed the proof.
    f.program.bodies.clear();
    incremental::with_checked(f.program, vec![proved], None, |token| {
        assert_eq!(token.program().bodies.len(), 1);
        assert_eq!(token.program().bodies[0].instance, Some(first));
    })?;
    Ok(())
}

#[test]
fn assembly_rejects_unproved_bodies_and_inconsistent_cleanup() -> TestResult {
    let f = fixture("fn main(){}")?;
    assert!(incremental::with_checked(f.program, vec![], None, |_| ()).is_err());
    let mut f = fixture("fn main(){}")?;
    f.program.bodies.clear();
    let recipes = body::drops::glue::Cache::new(&[])?.into_recipes();
    assert!(incremental::with_checked(f.program, vec![], Some(recipes), |_| ()).is_err());
    let mut f = fixture("struct D;impl Drop for D{fn drop(&mut self){}}fn main(){let d=D;}")?;
    f.program.bodies.clear();
    let recipes = body::drops::glue::Cache::new(&[])?.into_recipes();
    assert!(incremental::with_checked(f.program, vec![], Some(recipes), |_| ()).is_err());
    Ok(())
}

#[test]
fn assembly_cannot_erase_a_completed_destruction_proof() -> TestResult {
    let mut f = fixture("struct D;impl Drop for D{fn drop(&mut self){}}fn main(){let d=D;}")?;
    let checked = originals(&mut f.facts, &f.templates, &f.inputs)?;
    let (mut builder, bodies) = super::Builder::new(f.program)?;
    for body in bodies {
        let input = f
            .inputs
            .iter()
            .find(|input| input.key == body.key())
            .ok_or("missing ordinary contract")?;
        builder.push(&mut f.facts, &checked, body, input, None)?;
    }
    builder.program.drops = None;
    builder.recipes = None;
    let error = builder
        .with_checked(|_| ())
        .err()
        .ok_or("erased destruction was accepted")?;
    assert!(error.message.contains("inconsistent cleanup"));
    Ok(())
}

#[test]
fn unresolved_bodies_cannot_supply_callable_proof_results() -> TestResult {
    let mut f = fixture("fn id<T>(x:T)->T{x}fn main(){let _=id(7u8);}")?;
    let checked = originals(&mut f.facts, &f.templates, &f.inputs)?;
    let instance = *f.instances.first().ok_or("missing selected instance")?;
    let (mut body, contract) = specialize(&mut f.facts, &checked, instance)?;
    body.resolved_callables = Some(Box::default());
    let error = prove(&mut f.facts, &checked, body, &contract)
        .err()
        .ok_or("unproved target table was accepted")?;
    assert!(error.message.contains("completed body proof"));
    Ok(())
}

/// Compile `source` through demanded instances, either sharing each template's
/// substitution-invariant facts or re-proving them for every instance.
fn shared_lowir(source: &str, reprove: bool) -> TestResult<(Vec<u8>, usize, usize)> {
    let mut report = resolve_source(source)?;
    let mut facts =
        sema::declarations::analyze(&report).and_then(|f| sema::bodies::analyze(&report, f))?;
    let entry = sema::entry::select(&mut report, &facts)?;
    borrow::structure::REPROVE.set(reprove);
    borrow::structure::SHARED.set(0);
    borrow::structure::SPLIT.set(0);
    let program = crate::lower::instances::build(&report, &mut facts, entry);
    borrow::structure::REPROVE.set(false);
    let mut text = Vec::new();
    crate::lowir::writer::write(&program?, &mut text)?;
    Ok((
        text,
        borrow::structure::SHARED.get(),
        borrow::structure::SPLIT.get(),
    ))
}

#[test]
fn instances_share_template_structure_exactly_as_full_reproof() -> TestResult {
    // One template body each, demanded with Copy, non-Copy, Drop and reference
    // arguments. A promotable `&O::N` is promoted only where `O<T>` needs no
    // destructor, so `literal` has two structures for one template.
    let prefix = "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\
        #[derive(Clone,Copy)]struct P(u64);struct N(u64);\
        struct D<'a>(&'a mut u64,u64);impl Drop for D<'_>{fn drop(&mut self){*self.0+=self.1;}}\
        struct W<T>{v:T,n:u64}enum O<T>{S(T),N}\
        fn make<T>(v:T,n:u64)->W<T>{let w=W{v,n};if n>3{w}else{W{v:w.v,n:n+1}}}\
        fn count<T>(w:W<T>,k:u64)->u64{let mut i=0;let mut s=w.n;while i<k{if s>9{s=s-7;}else{s=s+i;}i=i+1;}let t=&s;*t}\
        fn pick<T>(a:T,b:T,c:bool)->T{let r=if c{a}else{b};let _q=&c;r}\
        fn first<'a,T>(x:&'a W<T>)->&'a T{&x.v}\
        fn literal<T>(x:T)->bool{let r=&O::<T>::N;let _x=x;match r{O::N=>true,O::S(_)=>false}}";
    let main = "fn main(){let mut dropped=0u64;let x=5u64;\
        let a=count(make(P(1),2),3);let b=count(make(N(1),2),3);let c=count(make(&x,2),3);\
        {let d=count(make(D(&mut dropped,1),2),3);check(d==6);}\
        let p=pick(P(1),P(2),true);let q=pick(N(1),N(2),false);let r=pick(&x,&x,true);\
        {let mut other=0u64;let e=pick(D(&mut dropped,2),D(&mut other,3),true);let _f=e.1;}\
        let w=make(N(9),5);let f=first(&w).0;let g=**first(&make(&x,5));\
        check(literal(P(1))&&literal(N(2))&&literal(&x)&&literal(D(&mut dropped,0)));\
        check(a==6&&b==6&&c==6&&p.0==1&&q.0==2&&*r==5&&f==9&&g==5);}";
    for (with_drop, source) in [
        (true, format!("{prefix}{main}")),
        (
            false,
            format!(
                "{prefix}fn main(){{let x=5u64;check(count(make(P(1),2),3)==count(make(N(1),2),3));check(count(make(&x,2),3)==6&&literal(P(1))&&literal(&x)&&pick(N(1),N(2),true).0==1&&*pick(&x,&x,false)==5);}}"
            ),
        ),
    ] {
        let (shared, reused, split) = shared_lowir(&source, false)?;
        let (reproved, none, _) = shared_lowir(&source, true)?;
        assert_eq!(none, 0, "the re-proof path shared facts");
        assert!(reused >= 2, "only {reused} instances shared facts");
        assert_eq!(split, usize::from(with_drop), "promotion keys");
        assert!(
            shared == reproved,
            "shared instance structure changed LowIR (destructors: {with_drop})"
        );
    }
    Ok(())
}
