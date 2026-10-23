//! Joint CFG proofs do not substitute for each original declaration identity.
use super::*;
use crate::{body, borrow, types};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn component_identity_preflight_precedes_every_hidden_head_publication() -> Result {
    for forged in [false, true] {
        let report = crate::test_source::resolve(
            "const fn a<'a>(x:&'a u8)->impl Copy+use<'a>{x} const fn b<'b>(x:&'b u8)->impl Copy+use<'b>{x} fn main(){}",
            true,
        )?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        let mut templates = body::templates::build(&report, &facts)?;
        assert_eq!(templates.bodies.len(), 2);
        assert!(
            facts
                .opaques
                .declarations
                .values()
                .all(|d| d.hidden.is_none())
        );
        let mut inputs = body::regions::collect(&facts)?.bodies;
        for input in &inputs {
            input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
        }
        if forged {
            // A same-number foreign source remains internally paired with its
            // contract. Only the actual original declaration check rejects it.
            let body = &mut templates.bodies[1].body;
            body.source = crate::resolve::unit::NodeKey {
                file: crate::resolve::unit::FileId(u32::MAX),
                ..crate::test_sites::node(body.source)?
            }
            .into();
            inputs
                .iter_mut()
                .find(|c| c.key == body.key())
                .ok_or("contract")?
                .source = body.source;
        }
        let checked = checked_family(&facts, &templates, inputs)?;
        let result = close_component(&report, &mut facts, &templates, &checked);
        if forged {
            assert_eq!(
                result.err().ok_or("accepted forged original")?.message,
                "original component changed its typed owner or output link"
            );
        } else {
            result?;
        }
        for declaration in facts.opaques.declarations.values() {
            assert_eq!(declaration.hidden.is_none(), forged);
            // Closing the source proof does not fabricate a physical layout.
            assert!(facts.types.storage_head(declaration.application).is_err());
        }
    }
    Ok(())
}

fn checked_family<'a>(
    facts: &Declarations,
    templates: &'a Templates,
    inputs: Vec<Contract>,
) -> Result<borrow::templates::Checked<'a>> {
    let mut copies = types::copy::Copies::default();
    Ok(borrow::templates::verify(
        &mut borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: facts.uses_source_copy(),
            copies: &mut copies,
        },
        &facts.lifetimes.variances,
        &body::regions::Contracts {
            bodies: inputs,
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        },
        templates,
        &[],
        |ty, field| body::drops::nominal_position(facts, ty, field),
    )?)
}
