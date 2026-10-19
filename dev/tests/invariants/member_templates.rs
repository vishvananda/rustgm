//! Inferred member shapes preserve inputs while the body retains formation debts.
use super::{BYTE, Fixture, Kind, RegionId, TestResult, TypeId, WORD, fixture};
use crate::{
    resolve::model::DefId,
    sema::traits::{
        assumptions::Assumptions,
        selection::{self, Caller},
    },
    types::{
        Primitive,
        arguments::Constant,
        infer::{Context, Domain, Error, TermId, arguments::Argument},
        lifetime_proof::Obligations,
    },
};
use std::fmt::Write as _;
const SIZE: TypeId = TypeId::primitive(Primitive::Usize);

// These internal tests exercise a selected definition's inference transport.
// Completing this inventory is deliberate setup, not source conformance evidence.
fn prepared(source: &str) -> TestResult<Fixture> {
    let mut f = fixture(source)?;
    let mut registry = f
        .facts
        .generics
        .implementations
        .take()
        .ok_or("missing inventory")?;
    let owners: Vec<_> = registry.headers().map(|h| h.owner).collect();
    for owner in owners {
        let types = f
            .facts
            .associated
            .type_definition(owner, DefId(f.template.item.0))
            .map(|member| vec![(DefId(f.template.item.0), member)])
            .unwrap_or_default();
        registry.complete(owner, Vec::new(), types, Vec::new())?;
    }
    f.facts.generics.implementations = Some(registry);
    Ok(f)
}
fn expose(
    f: &mut Fixture,
    context: &mut Context,
    root: TermId,
) -> TestResult<Option<(TermId, Obligations)>> {
    Ok(selection::symbolic_result(
        &mut f.facts,
        context,
        root,
        &Caller {
            assumptions: &Assumptions::default(),
            environment: None,
            first_region: 100,
            retain_sized: false,
        },
    )?)
}
fn projection(
    f: &Fixture,
    c: &mut Context,
    subject: TypeId,
    parent: &[Argument],
    own: &[Argument],
) -> TestResult<TermId> {
    let subject = c.known(&f.facts.types, subject)?;
    Ok(c.projected(&f.facts.types, f.template.item, subject, parent, own)?)
}
fn equate(f: &mut Fixture, c: &mut Context, inferred: TermId, expected: TypeId) -> TestResult {
    let known = c.known(&f.facts.types, expected)?;
    c.equate(&f.facts.types, inferred, known)?;
    Ok(())
}
fn tuple(f: &mut Fixture, values: &[TypeId]) -> TestResult<TypeId> {
    let values = f.facts.types.intern_list(values)?;
    Ok(f.facts.types.intern(Kind::Tuple(values))?)
}
fn reference(f: &mut Fixture, region: RegionId, referent: TypeId) -> TestResult<TypeId> {
    Ok(f.facts.types.intern(Kind::Reference {
        region,
        referent,
        mutable: false,
    })?)
}

#[test]
fn selected_shape_infers_parent_member_and_const_inputs_without_erasing_query() -> TestResult {
    let mut f = prepared(
        "trait Q<T>{type Out<U,const N:usize>;}impl<T>Q<T>for u8{type Out<U,const N:usize>=(T,U,[U;N]);}",
    )?;
    let mut c = Context::default();
    let parent = c.variable(Domain::Any)?;
    let own = c.variable(Domain::Any)?;
    let count = c.const_variable(&f.facts.types, SIZE)?;
    let root = projection(
        &f,
        &mut c,
        BYTE,
        &[Argument::Type(parent)],
        &[Argument::Type(own), Argument::Const(count)],
    )?;
    let (result, _) = expose(&mut f, &mut c, root)?.ok_or("missing inferred family result")?;
    let array = f.facts.types.intern(Kind::Array {
        element: BYTE,
        length: 3,
    })?;
    let expected = tuple(&mut f, &[WORD, BYTE, array])?;
    equate(&mut f, &mut c, result, expected)?;
    assert_eq!(c.resolved(&mut f.facts.types, parent)?, WORD);
    assert_eq!(c.resolved(&mut f.facts.types, own)?, BYTE);
    let three = f
        .facts
        .types
        .intern_constant(Constant { ty: SIZE, bits: 3 })?;
    assert_eq!(c.resolved_const(&mut f.facts.types, count)?, three);
    assert_eq!(
        c.projection_item(&f.facts.types, root)?,
        Some(f.template.item)
    );
    assert_eq!(
        c.projection_application(&f.facts.types, root)?
            .arguments
            .len(),
        2
    );
    Ok(())
}

#[test]
fn shared_parent_member_variables_stay_shared_and_distinct_calls_stay_independent() -> TestResult {
    let mut f = prepared("trait Q<T>{type Out<U>;}impl<T>Q<T>for u8{type Out<U>=(T,U);}")?;
    let mut c = Context::default();
    let a = c.variable(Domain::Any)?;
    let b = c.variable(Domain::Any)?;
    for (hole, expected) in [(a, BYTE), (b, WORD)] {
        let root = projection(
            &f,
            &mut c,
            BYTE,
            &[Argument::Type(hole)],
            &[Argument::Type(hole)],
        )?;
        let (result, _) = expose(&mut f, &mut c, root)?.ok_or("missing shared result")?;
        let first = c.child(&f.facts.types, result, 0)?;
        let second = c.child(&f.facts.types, result, 1)?;
        assert_eq!(c.root(first)?, c.root(second)?);
        let expected_tuple = tuple(&mut f, &[expected, expected])?;
        equate(&mut f, &mut c, result, expected_tuple)?;
        assert_eq!(c.resolved(&mut f.facts.types, hole)?, expected);
    }
    assert_ne!(c.root(a)?, c.root(b)?);
    Ok(())
}

#[test]
fn erased_output_does_not_bind_member_inputs_or_discharge_their_conditions() -> TestResult {
    let mut f = prepared(
        "trait Mark{}impl Mark for u8{}trait Q{type Out<T>where T:Mark;}impl Q for u8{type Out<T>=u8 where T:Mark;}",
    )?;
    let mut c = Context::default();
    let hole = c.variable(Domain::Any)?;
    let root = projection(&f, &mut c, BYTE, &[], &[Argument::Type(hole)])?;
    let (result, proof) = expose(&mut f, &mut c, root)?.ok_or("missing erased shape")?;
    assert_eq!(c.resolved(&mut f.facts.types, result)?, BYTE);
    assert_eq!(c.resolved(&mut f.facts.types, hole), Err(Error::Unresolved));
    assert!(proof.outlives.is_empty());
    equate(&mut f, &mut c, hole, WORD)?;
    f.template.subject = BYTE;
    let caller = f.environment(None)?;
    let selected = f.ready(&[super::Argument::Type(WORD)], caller)?;
    assert!(
        selected.source.is_none(),
        "an erased result must retain its failed member requirement"
    );
    Ok(())
}

#[test]
fn ambiguous_or_input_refining_headers_cannot_publish_shapes_or_bind_holes() -> TestResult {
    for implementation in [
        "impl Q<u8>for u8{type Out<T>=T;}impl Q<u16>for u8{type Out<T>=u8;}",
        "impl Q<u16>for u8{type Out<T>=T;}",
    ] {
        let mut f = prepared(&format!("trait Q<P>{{type Out<T>;}}{implementation}"))?;
        let mut c = Context::default();
        let parent = c.variable(Domain::Any)?;
        let own = c.variable(Domain::Any)?;
        let root = projection(
            &f,
            &mut c,
            BYTE,
            &[Argument::Type(parent)],
            &[Argument::Type(own)],
        )?;
        let parent_before = c.root(parent)?;
        let own_before = c.root(own)?;
        assert!(expose(&mut f, &mut c, root)?.is_none());
        assert_eq!(c.root(parent)?, parent_before);
        assert_eq!(c.root(own)?, own_before);
        assert_eq!(
            c.resolved(&mut f.facts.types, parent),
            Err(Error::Unresolved)
        );
        assert_eq!(c.resolved(&mut f.facts.types, own), Err(Error::Unresolved));
    }
    Ok(())
}

#[test]
fn independent_header_fast_path_shares_parent_and_member_input_holes() -> TestResult {
    let mut f = prepared("trait Q{type Out<U>;}impl<T>Q for T{type Out<U>=(T,U);}")?;
    let mut c = Context::default();
    let hole = c.variable(Domain::Any)?;
    let subject = c.construct(
        crate::types::infer::Constructor::Reference {
            region: RegionId::STATIC,
            mutable: false,
        },
        &[hole],
    )?;
    let root = c.projected(
        &f.facts.types,
        f.template.item,
        subject,
        &[],
        &[Argument::Type(hole)],
    )?;
    let (result, _) = expose(&mut f, &mut c, root)?.ok_or("missing independent-header result")?;
    let reference = reference(&mut f, RegionId::STATIC, WORD)?;
    let expected = tuple(&mut f, &[reference, WORD])?;
    equate(&mut f, &mut c, result, expected)?;
    assert_eq!(c.resolved(&mut f.facts.types, hole)?, WORD);
    Ok(())
}

#[test]
fn parent_member_and_callable_local_regions_keep_their_own_binders() -> TestResult {
    let mut f = prepared(
        "trait Q<'p>{type Out<'a,T>;}impl<'p>Q<'p>for u8{type Out<'a,T>=for<'b>fn(&'b T,&'p u8,&'a u8)->T;}",
    )?;
    let mut c = Context::default();
    let hole = c.variable(Domain::Any)?;
    let parent = RegionId::bound(0, 0)?;
    let own = RegionId::bound(0, 1)?;
    let root = projection(
        &f,
        &mut c,
        BYTE,
        &[Argument::Lifetime(parent)],
        &[Argument::Lifetime(own), Argument::Type(hole)],
    )?;
    let (result, _) = expose(&mut f, &mut c, root)?.ok_or("missing callable shape")?;
    equate(&mut f, &mut c, hole, WORD)?;
    let local = reference(&mut f, RegionId::bound(0, 0)?, WORD)?;
    let parent = reference(&mut f, RegionId::bound(1, 0)?, BYTE)?;
    let own = reference(&mut f, RegionId::bound(1, 1)?, BYTE)?;
    let parameters = f.facts.types.intern_list(&[local, parent, own])?;
    let expected = f.facts.types.intern(Kind::Function {
        parameters,
        result: WORD,
        bound_regions: 1,
        unsafe_: false,
    })?;
    assert_eq!(c.resolved(&mut f.facts.types, result)?, expected);
    Ok(())
}

fn width(count: usize) -> TestResult<usize> {
    let mut parameters = String::new();
    let mut values = String::new();
    for index in 0..count {
        write!(parameters, "T{index},")?;
        write!(values, "T{index},")?;
    }
    let mut f = prepared(&format!(
        "trait Q{{type Out<{parameters}>;}}impl Q for u8{{type Out<{parameters}>=({values});}}"
    ))?;
    let mut c = Context::default();
    let own = (0..count)
        .map(|_| c.variable(Domain::Any).map(Argument::Type))
        .collect::<Result<Vec<_>, _>>()?;
    let root = projection(&f, &mut c, BYTE, &[], &own)?;
    let before = c.steps;
    let (result, _) = expose(&mut f, &mut c, root)?.ok_or("missing wide shape")?;
    let work = c.steps - before;
    for (index, &arg) in own.iter().enumerate() {
        let Argument::Type(original) = arg else {
            return Err("missing own type".into());
        };
        let actual = c.child(&f.facts.types, result, index)?;
        assert_eq!(c.root(actual)?, c.root(original)?);
    }
    Ok(work)
}
#[test]
fn inferred_member_shape_work_scales_with_distinct_arguments() -> TestResult {
    let small = width(16)?;
    let medium = width(64)?;
    let large = width(256)?;
    eprintln!("symbolic family work: {small}, {medium}, {large}");
    assert!(medium < small * 5);
    assert!(large < medium * 5);
    Ok(())
}
