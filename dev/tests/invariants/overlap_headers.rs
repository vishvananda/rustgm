//! Head disjointness avoids unrelated conditional graphs without dropping real debt.
use super::*;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn work(width: usize, same_arguments: bool) -> Result<usize> {
    let mut source = String::from("trait A<U>{}struct End;");
    for index in 0..width {
        write!(source, "trait C{index}{{}}")?;
    }
    source.push_str("impl<T:");
    for index in 0..width {
        if index != 0 {
            source.push('+');
        }
        write!(source, "C{index}")?;
    }
    write!(
        source,
        ">A<[u8;0]>for T{{}}impl A<[u8;{}]>for End{{}}",
        usize::from(!same_arguments)
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let declaration = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| report.package.names.get(definition.name) == "A")
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or("actual trait A")?;
    let pairs: Vec<_> = facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .headers()
        .filter(|header| {
            header
                .predicate
                .bound
                .trait_parts()
                .and_then(|(id, _)| facts.types.trait_application(id))
                .is_some_and(|application| application.item.0 == declaration.0)
        })
        .map(|header| (header.predicate, header.owner))
        .collect();
    let [first, second] = pairs.as_slice() else {
        return Err("actual pair of A headers".into());
    };
    let (intersects, work) = possible(&mut facts, *first, *second)?;
    assert!(
        !intersects,
        "closed local End has none of the required C implementations"
    );
    Ok(work)
}

#[test]
fn disjoint_heads_skip_condition_expansion_while_compatible_heads_pay_real_conditions() -> Result {
    let mut disjoint = Vec::new();
    let mut conditional = Vec::new();
    for width in [8, 32, 128] {
        disjoint.push(work(width, false)?);
        conditional.push(work(width, true)?);
    }
    assert!(
        disjoint
            .iter()
            .all(|&work| work <= disjoint[0].saturating_mul(2)),
        "{disjoint:?}"
    );
    assert!(
        conditional.windows(2).all(|pair| pair[0] < pair[1]),
        "{conditional:?}"
    );
    println!("disjoint head work{disjoint:?}, retained condition work{conditional:?}");
    Ok(())
}
