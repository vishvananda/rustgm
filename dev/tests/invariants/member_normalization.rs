//! Nested selection transports complete member inputs and their formation evidence.
use super::{
    Abilities, Application, Arc, Argument, BYTE, Bound, Environment, Fixture, GenericArgs, ItemId,
    Kind, Predicate, RegionId, TestResult, TypeId, WORD, fixture,
};
use crate::{
    sema::traits::{
        assumptions::Assumptions,
        heads::Head,
        implementations::{Availability, Implementations, PendingHeader},
        selection::{self, Caller, Selection},
    },
    types::{
        infer::{Context, Instantiation},
        parameters::clauses::{Clause, Quantified},
        traits,
    },
};
use std::fmt::Write as _;

impl Fixture {
    fn projection(&mut self, own: &[Argument]) -> TestResult<TypeId> {
        let arguments = self.facts.types.intern_arguments(own)?;
        Ok(self.facts.types.intern(Kind::Projection(Application {
            arguments,
            ..self.template
        }))?)
    }
    fn parent(&self) -> Predicate {
        Predicate {
            ty: self.template.subject,
            bound: Bound::Trait(self.template.trait_application),
        }
    }
    fn caller(&self, clauses: &[Clause]) -> TestResult<Arc<Environment>> {
        Ok(Arc::new(
            Environment::new(&self.facts.types, &[(self.parameter, Abilities::default())])?
                .with_clauses(&self.facts.types, clauses)?,
        ))
    }
    fn mark(&mut self, ty: TypeId) -> TestResult<Predicate> {
        Ok(Predicate {
            ty,
            bound: Bound::Trait(self.facts.types.intern_trait_application(
                traits::Application {
                    item: ItemId(self.mark.ok_or("missing Mark")?.0),
                    arguments: GenericArgs::EMPTY,
                },
            )?),
        })
    }
    fn select(
        &mut self,
        goal: Predicate,
        caller: &Arc<Environment>,
    ) -> TestResult<(Selection, usize)> {
        let mut context = Context::default();
        let mut substitution = Instantiation::new(&mut context, &self.facts.types, &[])?;
        let goal = context.instantiate_predicate(&mut self.facts.types, goal, &mut substitution)?;
        let (assumptions, _) = Assumptions::from_environment(&self.facts.types, Some(caller))?;
        let selected = selection::select_declaration(
            &mut self.facts,
            &mut context,
            &goal,
            Caller {
                assumptions: &assumptions,
                environment: Some(caller),
                first_region: 100,
                retain_sized: false,
            },
        )?;
        Ok((selected, context.steps))
    }
    fn forall(&mut self, predicate: Predicate, bound_regions: u32) -> TestResult<Clause> {
        Ok(Clause::ForAll(self.facts.types.intern_quantified(
            Quantified {
                bound_regions,
                predicate,
            },
        )?))
    }
    fn function(&mut self, result: TypeId, bound_regions: u32) -> TestResult<TypeId> {
        let mut parameters = Vec::new();
        for index in 0..bound_regions {
            parameters.push(self.facts.types.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(0, u16::try_from(index)?)?,
                referent: BYTE,
            })?);
        }
        let parameters = self.facts.types.intern_list(&parameters)?;
        Ok(self.facts.types.intern(Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?)
    }
}

fn equal(ty: TypeId, result: TypeId) -> Predicate {
    Predicate {
        ty,
        bound: Bound::ProjectionEqual(result),
    }
}

#[test]
fn nested_selection_matches_member_arguments_before_using_an_equality() -> TestResult {
    let mut f = fixture("trait Mark{}impl Mark for u8{}trait Q{type Out<T>;}")?;
    let byte = f.projection(&[Argument::Type(BYTE)])?;
    let word = f.projection(&[Argument::Type(WORD)])?;
    let caller = f.caller(&[
        f.parent().into(),
        equal(byte, BYTE).into(),
        equal(word, WORD).into(),
    ])?;
    for (root, expected) in [(byte, true), (word, false)] {
        let goal = f.mark(root)?;
        let (selected, _) = f.select(goal, &caller)?;
        assert_eq!(matches!(selected, Selection::Proven(_)), expected);
        if let Selection::Proven(evidence) = selected {
            assert!(f.prove(&evidence.proof, 0, false)?);
        }
    }
    Ok(())
}

#[test]
fn nested_normalization_cannot_erase_a_member_requirement() -> TestResult {
    for extra in [None, Some(WORD)] {
        let mut f = fixture("trait Mark{}impl Mark for u8{}trait Q{type Out<T> where T:Mark;}")?;
        let inner = f.projection(&[Argument::Type(BYTE)])?;
        let normalized_outer = f.projection(&[Argument::Type(WORD)])?;
        let outer = f.projection(&[Argument::Type(inner)])?;
        let mut clauses = vec![
            f.parent().into(),
            equal(inner, WORD).into(),
            equal(normalized_outer, BYTE).into(),
        ];
        if let Some(ty) = extra {
            clauses.push(f.mark(ty)?.into());
        }
        let caller = f.caller(&clauses)?;
        let goal = f.mark(outer)?;
        let (selected, _) = f.select(goal, &caller)?;
        assert_eq!(matches!(selected, Selection::Proven(_)), extra.is_some());
        if let Selection::Proven(evidence) = selected {
            assert!(f.prove(&evidence.proof, 0, false)?);
        }
    }
    Ok(())
}

#[test]
fn callable_family_arguments_share_parent_opening_and_preserve_lifetime_correlations() -> TestResult
{
    for (region, expected) in [("'a", true), ("'static", false)] {
        let mut f = fixture(&format!(
            "trait Mark{{}}impl Mark for for<'a>fn(&'a u8)->&{region} u8{{}}trait Q<'p>{{type Out<'a,T>;}}"
        ))?;
        let local = RegionId::bound(0, 0)?;
        let output = f.projection(&[Argument::Lifetime(local), Argument::Type(BYTE)])?;
        let reference = f.facts.types.intern(Kind::Reference {
            mutable: false,
            region: local,
            referent: BYTE,
        })?;
        let parent = f.forall(f.parent(), 1)?;
        let equation = f.forall(equal(output, reference), 1)?;
        let caller = f.caller(&[parent, equation])?;
        let function = f.function(output, 1)?;
        let goal = f.mark(function)?;
        let (selected, _) = f.select(goal, &caller)?;
        let Selection::Proven(evidence) = selected else {
            assert!(!expected);
            continue;
        };
        assert!(evidence.proof.scope_count() > 0);
        assert_eq!(f.prove(&evidence.proof, 0, false)?, expected);
    }
    Ok(())
}

#[test]
fn erased_callable_family_result_retains_own_lifetime_requirements() -> TestResult {
    let mut f = fixture(
        "trait Mark{}impl Mark for for<'a>fn(&'a u8)->u8{}trait Q{type Out<'a> where 'a:'static;}",
    )?;
    let local = RegionId::bound(0, 0)?;
    let output = f.projection(&[Argument::Lifetime(local)])?;
    let equation = f.forall(equal(output, BYTE), 1)?;
    let caller = f.caller(&[f.parent().into(), equation])?;
    let function = f.function(output, 1)?;
    let goal = f.mark(function)?;
    let (selected, _) = f.select(goal, &caller)?;
    let Selection::Proven(evidence) = selected else {
        return Err("missing conditional family proof".into());
    };
    assert!(evidence.proof.scope_count() > 0);
    assert!(!f.prove(&evidence.proof, 0, false)?);
    Ok(())
}

#[test]
fn cyclic_member_requirements_stop_without_becoming_completed_trait_evidence() -> TestResult {
    let mut f = fixture("trait Mark{}impl Mark for u8{}trait Q{type Out<T>;}")?;
    let root = f.projection(&[Argument::Type(BYTE)])?;
    let condition = f.mark(root)?;
    let member = super::DefId(f.template.item.0);
    f.facts
        .associated
        .families
        .get_mut(&member)
        .ok_or("missing family")?
        .requirements = Some(vec![condition.into()]);
    let caller = f.caller(&[f.parent().into(), equal(root, BYTE).into()])?;
    let (selected, steps) = f.select(condition, &caller)?;
    assert!(!matches!(selected, Selection::Proven(_)));
    assert!(
        steps < 2_000,
        "cyclic family conditions did not stop: {steps}"
    );
    f.facts
        .associated
        .families
        .get_mut(&member)
        .ok_or("missing family")?
        .requirements = Some(Vec::new());
    let (selected, _) = f.select(condition, &caller)?;
    assert!(matches!(selected, Selection::Proven(_)));
    Ok(())
}

#[test]
fn growing_family_argument_lists_keep_formation_and_selection_work_linear() -> TestResult {
    let mut previous = None;
    for width in [16, 64, 256] {
        let mut source = String::from("trait Mark{}impl Mark for u8{}trait Q{type Out<");
        for index in 0..width {
            write!(source, "T{index}:Mark,")?;
        }
        source.push_str(">;}");
        let mut f = fixture(&source)?;
        let root = f.projection(&vec![Argument::Type(BYTE); width])?;
        let caller = f.caller(&[f.parent().into(), equal(root, BYTE).into()])?;
        let goal = f.mark(root)?;
        let (selected, steps) = f.select(goal, &caller)?;
        let Selection::Proven(evidence) = selected else {
            return Err("growing family argument list was not proved".into());
        };
        assert!(f.prove(&evidence.proof, 0, false)?);
        eprintln!("family_arguments={width} selection_work={steps}");
        if let Some(previous) = previous {
            assert!(
                steps <= previous * 5,
                "member conditions rescan the argument list"
            );
        }
        previous = Some(steps);
    }
    Ok(())
}

#[test]
fn selected_family_definitions_preserve_distinct_callable_parent_and_member_lifetimes() -> TestResult
{
    for (second, expected) in [("'b", true), ("'a", false)] {
        let (mut f, goal) = concrete_goal(second)?;
        let caller = f.caller(&[])?;
        let (selected, _) = f.select(goal, &caller)?;
        let Selection::Proven(evidence) = selected else {
            assert!(!expected);
            continue;
        };
        assert!(evidence.proof.scope_count() > 0);
        assert_eq!(f.prove(&evidence.proof, 0, false)?, expected);
    }
    Ok(())
}

fn concrete_goal(second: &str) -> TestResult<(Fixture, Predicate)> {
    let mut f = fixture(&format!(
        "trait Mark{{}}impl Mark for for<'a,'b>fn(&'a u8,&'b u8)->(&'a u8,&{second} u8,u16){{}}
         trait Q<'p>{{type Out<'a,T>;}}impl<'p>Q<'p>for u8{{type Out<'a,T>=(&'p u8,&'a u8,T);}}"
    ))?;
    f.template.subject = BYTE;
    let result = f.projection(&[
        Argument::Lifetime(RegionId::bound(0, 1)?),
        Argument::Type(WORD),
    ])?;
    let function = f.function(result, 2)?;
    let goal = f.mark(function)?;
    Ok((f, goal))
}

#[test]
fn nested_member_applications_survive_header_waits_without_reopening_callable_scopes() -> TestResult
{
    let (mut f, predicate) = concrete_goal("'b")?;
    let registry = f
        .facts
        .generics
        .implementations
        .take()
        .ok_or("missing registry")?;
    let mut seeds = Vec::new();
    for header in registry.headers() {
        let Bound::Trait(id) = header.predicate.bound else {
            return Err("missing implementation trait".into());
        };
        let item = f
            .facts
            .types
            .trait_application(id)
            .ok_or("missing trait application")?
            .item;
        let kind = f
            .facts
            .types
            .kind(header.predicate.ty)
            .ok_or("missing implementation type")?;
        seeds.push(PendingHeader {
            owner: header.owner,
            item: item.into(),
            head: Some(Head::canonical(kind)),
        });
    }
    let mut pending = Implementations::default();
    pending.reserve(seeds)?;
    f.facts.generics.implementations = Some(Box::new(pending));
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &f.facts.types, &[])?;
    let goal = context.instantiate_predicate(&mut f.facts.types, predicate, &mut substitution)?;
    let selection::Progress::Pending(owner, mut query) = selection::begin(
        &mut f.facts,
        &mut context,
        &goal,
        selection::OwnedCaller {
            assumptions: Arc::new(Assumptions::default()),
            environment: None,
            first_region: 100,
            retain_sized: false,
        },
        Availability::Declarations,
    )?
    else {
        return Err("nested family query did not wait for its header".into());
    };
    let end = query.allocation_end();
    assert!(end >= 102);
    let before = (context.steps, f.facts.types.len());
    for _ in 0..4 {
        let selection::Progress::Pending(waiting, retained) =
            query.resume(&mut f.facts, &mut context, &goal, end)?
        else {
            return Err("nested family wait lost its continuation".into());
        };
        assert_eq!(waiting, owner);
        assert_eq!(retained.allocation_end(), end);
        assert_eq!((context.steps, f.facts.types.len()), before);
        query = retained;
    }
    f.facts.generics.implementations = Some(registry);
    let selection::Progress::Ready(Selection::Proven(evidence)) =
        query.resume(&mut f.facts, &mut context, &goal, end + 7)?
    else {
        return Err("prepared nested family query did not finish".into());
    };
    assert!(f.prove(&evidence.proof, 0, false)?);
    for id in end..end + 7 {
        assert!(!evidence.proof.owns_region(RegionId(id)));
    }
    Ok(())
}

#[test]
fn distinct_family_applications_share_their_completed_parent_trait_proof() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = String::from("trait Q{type Out<T>;}trait Mark{}impl Mark for (");
        source.push_str(&"u8,".repeat(width));
        source.push_str("){}");
        for index in 0..width {
            write!(source, "trait P{index}{{}}impl P{index} for u8{{}}")?;
        }
        source.push_str("impl Q for u8 where u8:");
        for index in 0..width {
            if index != 0 {
                source.push('+');
            }
            write!(source, "P{index}")?;
        }
        source.push_str("{type Out<T>=u8;}");
        let mut f = fixture(&source)?;
        f.template.subject = BYTE;
        let mut roots = Vec::new();
        for index in 0..width {
            let array = f.facts.types.intern(Kind::Array {
                element: BYTE,
                length: u64::try_from(index)?,
            })?;
            roots.push(f.projection(&[Argument::Type(array)])?);
        }
        let list = f.facts.types.intern_list(&roots)?;
        let root = f.facts.types.intern(Kind::Tuple(list))?;
        let goal = f.mark(root)?;
        let caller = f.caller(&[])?;
        let (selected, steps) = f.select(goal, &caller)?;
        assert!(matches!(selected, Selection::Proven(_)));
        eprintln!("family_applications={width} parent_conditions={width} selection_work={steps}");
        if let Some(previous) = previous {
            assert!(
                steps <= previous * 5,
                "distinct members repeated their parent proof"
            );
        }
        previous = Some(steps);
    }
    Ok(())
}
