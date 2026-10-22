//! Paid premise seals remain exact to caller epoch, context, availability and target.
use super::*;
use crate::{resolve::model::DefId, types::GenericArgs};
use catalog::{CallerKey, Enclosing, Publication};
#[test]
fn paid_owners_reject_cross_epoch_instance_availability_and_target_reuse()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = facts.functions[0];
    let source = function.body.ok_or("body")?;
    let key = CallerKey {
        source,
        declaration: Some(function.definition),
        universals: 0,
        enclosing: Enclosing::Original,
    };
    let publication = Publication::new([0..0, 0..0])?;
    let selected = Instance {
        definition: DefId(70),
        arguments: GenericArgs::EMPTY,
    };
    let mut first = catalog::Catalog::new()?;
    let caller = first.caller(key, publication)?;
    let paid = Paid {
        owner: first.caller_owner(caller)?,
        availability: Availability::Validated,
        selected,
    };
    let mut foreign = catalog::Catalog::new()?;
    let foreign_caller = foreign.caller(key, publication)?;
    let foreign_paid = Paid {
        owner: foreign.caller_owner(foreign_caller)?,
        ..paid
    };
    let other_key = CallerKey {
        enclosing: Enclosing::Initializer(Instance {
            definition: function.definition,
            arguments: GenericArgs(77),
        }),
        ..key
    };
    let other = first.caller(other_key, publication)?;
    facts.source_computations = Some(Box::new(super::super::Computations(first)));
    paid.validate(&facts, caller, key, selected)?;
    for _ in 0..128 {
        paid.validate(&facts, caller, key, selected)?;
        assert!(paid.validate(&facts, other, other_key, selected).is_err());
        assert!(
            foreign_paid
                .validate(&facts, caller, key, selected)
                .is_err()
        );
        assert!(
            Paid {
                availability: Availability::Declarations,
                ..paid
            }
            .validate(&facts, caller, key, selected)
            .is_err()
        );
        assert!(
            paid.validate(
                &facts,
                caller,
                key,
                Instance {
                    arguments: GenericArgs(1),
                    ..selected
                }
            )
            .is_err()
        );
        assert!(
            paid.validate(
                &facts,
                caller,
                CallerKey {
                    universals: 1,
                    ..key
                },
                selected
            )
            .is_err()
        );
    }
    let catalog = &mut facts.source_computations.as_deref_mut().ok_or("catalog")?.0;
    assert!(
        catalog
            .caller(key, Publication::new([0..0, 0..1])?)
            .is_err()
    );
    eprintln!(
        "paid layouts inputs={} caller={} frame={} seal={} use={} catalog={} dispatcher={} target={} request={} code={}",
        size_of::<Option<Inputs<'_>>>(),
        size_of::<crate::sema::values::execution::BodyCaller<'_>>(),
        size_of::<crate::sema::constants::pending_patterns::Frame>(),
        size_of::<super::super::seals::Seal>(),
        size_of::<catalog::Use>(),
        size_of::<catalog::Catalog>(),
        size_of::<crate::sema::dispatch::Dispatcher>(),
        size_of::<crate::sema::values::execution::Target>(),
        size_of::<crate::sema::values::execution::Request>(),
        size_of::<crate::const_eval::code::Code>()
    );
    Ok(())
}
