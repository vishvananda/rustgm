//! Parent inference publishes only proved inputs and keeps member normalization pending.
use super::{Attempt, Context, Declarations, Opened};
use crate::{
    resolve::load::{InputFile, Sources},
    sema::{
        generics::Parameter,
        traits::{
            assumptions::Assumptions,
            implementations::Availability,
            selection::{self, Caller},
        },
    },
    types::{
        ItemId, Kind, Primitive, TypeId,
        arguments::Argument as Canonical,
        infer::{Domain, Error, TermId, arguments::Argument},
        parameters::{
            Parameter as Formal, ParameterKind,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        projection::Application,
    },
};
use std::sync::Arc;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const WORD: TypeId = TypeId::primitive(Primitive::U16);
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
#[expect(
    clippy::default_trait_access,
    reason = "resolution fixture uses standard expansion defaults"
)]
fn facts(implementation: &str) -> TestResult<(Declarations, ItemId)> {
    let source = format!("trait Q<P>{{type Out;}}{implementation}");
    let input = InputFile {
        identity: "inputs.rs".into(),
        path: "inputs.rs".into(),
        source: crate::support::source::SourceFile::new(source.into_bytes())?,
    };
    let expanded = crate::resolve::expansion::analyze(input, &mut NoFiles, &Default::default())?;
    let report = crate::resolve::analyze_expanded(expanded)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let members: Vec<_> = facts
        .generics
        .implementations()
        .flat_map(|i| i.types.iter().copied())
        .collect();
    let declared = members.first().ok_or("missing member")?.0;
    // Install already-declared family identities at this inference boundary.
    // Source family conformance is deliberately outside this transport fixture.
    for (_, member) in members {
        let id = facts.types.intern_parameter(Formal {
            owner: ItemId(member.0),
            index: 0,
            kind: ParameterKind::Type { sized: true },
        })?;
        let template = facts.types.intern(Kind::Parameter(id))?;
        facts.generics.parameters.insert(
            member,
            vec![Parameter {
                anonymous: false,
                binding: member,
                argument: Canonical::Type(template),
                default: None,
            }],
        );
        facts.definitions[member.0 as usize] = Some(template);
    }
    Ok((facts, ItemId(declared.0)))
}
fn root(
    facts: &Declarations,
    c: &mut Context,
    item: ItemId,
    parent: TermId,
    own: TermId,
) -> TestResult<TermId> {
    let subject = c.known(&facts.types, BYTE)?;
    Ok(c.projected(
        &facts.types,
        item,
        subject,
        &[Argument::Type(parent)],
        &[Argument::Type(own)],
    )?)
}
#[test]
fn proved_parent_refines_before_member_output_and_never_completes_the_query() -> TestResult {
    let (mut facts, item) = facts("impl Q<u16>for u8{type Out=u8;}")?;
    let mut c = Context::default();
    let parent = c.variable(Domain::Any)?;
    let own = c.variable(Domain::Any)?;
    let root = root(&facts, &mut c, item, parent, own)?;
    let mut opened = Opened::new(&mut facts, &mut c, root, None, 0, 100)?;
    let Attempt::Retry(Some(inputs)) = opened.select(&mut facts, Availability::Validated)? else {
        return Err("missing parent refinement".into());
    };
    assert_eq!(c.resolved(&mut facts.types, parent), Err(Error::Unresolved));
    assert!(inputs.arguments.is_none());
    assert!(inputs.commit(&mut facts, &mut c, root, &[])?);
    assert_eq!(c.resolved(&mut facts.types, parent)?, WORD);
    assert_eq!(c.resolved(&mut facts.types, own), Err(Error::Unresolved));
    assert_eq!(c.projection_item(&facts.types, root)?, Some(item));
    let mut again = Opened::new(&mut facts, &mut c, root, None, 0, 200)?;
    let Attempt::Retry(Some(inputs)) = again.select(&mut facts, Availability::Validated)? else {
        return Err("missing unchanged inputs".into());
    };
    assert!(!inputs.commit(&mut facts, &mut c, root, &[])?);
    let (result, _) = selection::symbolic_result(
        &mut facts,
        &mut c,
        root,
        &Caller {
            assumptions: &Assumptions::default(),
            environment: None,
            first_region: 300,
            retain_sized: false,
        },
    )?
    .ok_or("parent refinement did not enable the member shape")?;
    let byte = c.known(&facts.types, BYTE)?;
    c.equate(&facts.types, result, byte)?;
    assert_eq!(c.resolved(&mut facts.types, own)?, BYTE);
    assert_eq!(c.projection_item(&facts.types, root)?, Some(item));
    Ok(())
}
#[test]
fn ambiguous_parent_trials_leave_both_input_groups_unconstrained() -> TestResult {
    let (mut facts, item) = facts("impl Q<u8>for u8{type Out=u8;}impl Q<u16>for u8{type Out=u8;}")?;
    let mut c = Context::default();
    let parent = c.variable(Domain::Any)?;
    let own = c.variable(Domain::Any)?;
    let root = root(&facts, &mut c, item, parent, own)?;
    let mut opened = Opened::new(&mut facts, &mut c, root, None, 0, 100)?;
    assert!(matches!(
        opened.select(&mut facts, Availability::Validated)?,
        Attempt::Retry(None)
    ));
    assert_eq!(c.resolved(&mut facts.types, parent), Err(Error::Unresolved));
    assert_eq!(c.resolved(&mut facts.types, own), Err(Error::Unresolved));
    Ok(())
}
#[test]
fn projections_nested_in_own_arguments_wait_for_input_normalization() -> TestResult {
    let (mut facts, item) = facts("impl Q<u16>for u8{type Out=u8;}")?;
    let mut c = Context::default();
    let parent = c.known(&facts.types, WORD)?;
    let own = c.variable(Domain::Any)?;
    let inner = root(&facts, &mut c, item, parent, own)?;
    let outer = root(&facts, &mut c, item, parent, inner)?;
    let mut opened = Opened::new(&mut facts, &mut c, outer, None, 0, 100)?;
    assert!(matches!(
        opened.select(&mut facts, Availability::Validated)?,
        Attempt::NestedInputs
    ));
    assert_eq!(c.resolved(&mut facts.types, own), Err(Error::Unresolved));
    Ok(())
}
#[test]
fn caller_equality_can_refine_member_inputs_without_completing_formation() -> TestResult {
    let (mut facts, item) = facts("impl Q<u16>for u8{type Out=u8;}")?;
    let schema = facts
        .types
        .projection_schema(item)
        .ok_or("missing schema")?;
    let arguments = facts.types.intern_arguments(&[Canonical::Type(WORD)])?;
    let trait_application =
        facts
            .types
            .intern_trait_application(crate::types::traits::Application {
                item: schema.trait_item,
                arguments,
            })?;
    let own = facts.types.intern_arguments(&[Canonical::Type(BYTE)])?;
    let projected = facts.types.intern(Kind::Projection(Application {
        item,
        subject: BYTE,
        trait_application,
        arguments: own,
    }))?;
    let environment = Arc::new(
        Environment::new(&facts.types, &[])?.with_clauses(
            &facts.types,
            &[
                Predicate {
                    ty: BYTE,
                    bound: Bound::Trait(trait_application),
                }
                .into(),
                Predicate {
                    ty: projected,
                    bound: Bound::ProjectionEqual(WORD),
                }
                .into(),
            ],
        )?,
    );
    let mut c = Context::default();
    let parent = c.known(&facts.types, WORD)?;
    let hole = c.variable(Domain::Any)?;
    let root = root(&facts, &mut c, item, parent, hole)?;
    let mut opened = Opened::new(&mut facts, &mut c, root, Some(&environment), 0, 100)?;
    let Attempt::Retry(Some(inputs)) = opened.select(&mut facts, Availability::Validated)? else {
        return Err("missing complete equality inputs".into());
    };
    assert_eq!(c.resolved(&mut facts.types, hole), Err(Error::Unresolved));
    assert!(inputs.arguments.is_some());
    assert!(inputs.commit(&mut facts, &mut c, root, &[])?);
    assert_eq!(c.resolved(&mut facts.types, hole)?, BYTE);
    assert_eq!(c.projection_item(&facts.types, root)?, Some(item));
    Ok(())
}

#[test]
fn complete_member_input_matching_keeps_ambient_lifetime_slots_rigid() -> TestResult {
    use crate::types::{RegionId, infer::Constructor};
    for (slot, accepted) in [(0, true), (1, false)] {
        let (mut facts, item) = facts("impl Q<u16>for u8{type Out=u8;}")?;
        let mut c = Context::default();
        let hole = c.variable(Domain::Any)?;
        let own = c.construct(
            Constructor::Reference {
                region: RegionId::bound(0, 0)?,
                mutable: false,
            },
            &[hole],
        )?;
        let parent = c.known(&facts.types, WORD)?;
        let root = root(&facts, &mut c, item, parent, own)?;
        let original = c.projection_application(&facts.types, root)?;
        let predicate = c.resolved_predicate(&mut facts.types, &original.predicate)?;
        let ty = facts.types.intern(Kind::Reference {
            region: RegionId::bound(0, slot)?,
            mutable: false,
            referent: BYTE,
        })?;
        let arguments = Some(facts.types.intern_arguments(&[Canonical::Type(ty)])?);
        let inputs = super::FamilyInputs {
            predicate,
            arguments,
        };
        let result = inputs.commit(&mut facts, &mut c, root, &[2]);
        if accepted {
            assert!(result?);
            assert_eq!(c.resolved(&mut facts.types, hole)?, BYTE);
            assert!(c.region_equalities.is_empty());
        } else {
            assert_eq!(result, Err(Error::Mismatch));
        }
    }
    Ok(())
}
