//! Exact canonical prefixes precede implicit Self and elaborated assumptions.
use super::{Header, Trait, authenticate};
use crate::{
    resolve::expansion::Report,
    sema::{declaration_inputs::Inputs, model::Declarations},
    support::{bindings::LangItem, crates::MAX_COMPONENTS},
    types::{
        ItemId,
        parameters::{clauses::Clause, predicates::Bound},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
struct Visits {
    steps: usize,
}
impl Visits {
    fn charge(&mut self, count: usize) -> std::result::Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > MAX_COMPONENTS {
            return Err("test prefix work budget exceeded");
        }
        Ok(())
    }
}

fn prepared() -> Result<(Report, Declarations)> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "trait_prefixes")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn view<'a>(
    report: &Report,
    facts: &'a Declarations,
    role: LangItem,
) -> Result<(Trait<'a>, &'a Header)> {
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .header_role(role)?
        .ok_or("role")?;
    let header = Trait::source(report, facts, declaration)?;
    let canonical = facts
        .generics
        .traits
        .as_deref()
        .and_then(|traits| traits.header(ItemId(declaration.definition.0)))
        .ok_or("canonical")?;
    Ok((header, canonical))
}
fn copied(header: &Header) -> Result<Header> {
    let source = header.source_inputs()?;
    Ok(Header {
        self_type: header.self_type,
        parameters: header.parameters.clone(),
        inputs: crate::sema::traits::inherited_inputs::HeaderInputs::source(
            source.self_bounds.clone(),
            source.predicates.clone(),
        ),
        results: header.results.clone(),
        regions: header.regions.clone(),
    })
}

#[test]
fn real_generated_self_defaults_are_distinct_from_literal_supertraits() -> Result {
    let (report, facts) = prepared()?;
    let meta = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .header_role(LangItem::MetaSized)?
        .ok_or("MetaSized")?
        .definition;
    for (role, literal, requirements) in [(LangItem::Clone, 1, 2), (LangItem::Freeze, 0, 0)] {
        let (header, canonical) = view(&report, &facts, role)?;
        let mut work = Visits::default();
        let prefix = authenticate(&facts, &header, canonical, |steps| work.charge(steps))?;
        assert_eq!(header.written.supertraits, literal);
        assert_eq!(canonical.source_inputs()?.predicates.len(), requirements);
        assert!(work.steps > requirements);
        assert!(prefix.canonical < prefix.declared);
        let has_meta = canonical.source_inputs()?.predicates.iter().any(|input| {
            matches!(input.value,
            Clause::Predicate(predicate) if predicate.bound.trait_parts().is_some_and(|(id, _)|
                facts.types.trait_application(id).is_some_and(|app| app.item.0 == meta.0)))
        });
        assert_eq!(has_meta, role != LangItem::Freeze);
        if role == LangItem::Clone {
            assert!(
                canonical
                    .source_inputs()?
                    .predicates
                    .iter()
                    .any(|input| matches!(input.value,
                Clause::Predicate(predicate) if predicate.bound == Bound::Sized))
            );
        }
    }
    Ok(())
}

#[test]
fn shifted_truncated_and_forged_canonical_prefixes_reject_exactly() -> Result {
    let (report, facts) = prepared()?;
    let (header, canonical) = view(&report, &facts, LangItem::Clone)?;
    let mut variants = Vec::new();
    let mut truncated = copied(canonical)?;
    truncated.source_inputs_mut()?.predicates.pop();
    variants.push(truncated);
    let mut shifted = copied(canonical)?;
    shifted.source_inputs_mut()?.predicates.rotate_left(1);
    variants.push(shifted);
    let implicit =
        facts.generics.predicates[&header.definition][canonical.source_inputs()?.predicates.len()];
    let mut forged = copied(canonical)?;
    let generated = forged
        .source_inputs_mut()?
        .predicates
        .iter_mut()
        .find(|input| input.source == implicit.source)
        .ok_or("generated default with shared source")?;
    // Same exact source and Self type, but the wrong actual trait application.
    generated.value = implicit.value;
    variants.push(forged);
    for variant in variants {
        let mut work = Visits::default();
        assert!(authenticate(&facts, &header, &variant, |steps| work.charge(steps)).is_err());
        assert!(work.steps > 0);
    }
    Ok(())
}

#[test]
fn later_inputs_must_retain_the_actual_complete_prefix_and_environment() -> Result {
    let (report, facts) = prepared()?;
    let (header, canonical) = view(&report, &facts, LangItem::Clone)?;
    let full = facts.generics.predicates[&header.definition].clone();
    for mut changed in [
        full[..canonical.source_inputs()?.predicates.len() - 1].to_vec(),
        full.clone(),
    ] {
        if changed.len() == full.len() {
            changed.rotate_left(1);
        }
        let (mut altered, _) = view(&report, &facts, LangItem::Clone)?;
        altered.inputs = Inputs::new(&changed, &[]);
        let mut work = Visits::default();
        assert!(authenticate(&facts, &altered, canonical, |steps| work.charge(steps)).is_err());
        assert!(work.steps > 0);
    }
    Ok(())
}
