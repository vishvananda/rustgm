//! Entire typed value/dependency storage is checked even when no statement reaches it.
use super::*;
use crate::{
    body::ir::Value,
    types::{Kind, arguments::Argument},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn unused_generic_dependencies_and_values_are_not_independent_executable_footprints() -> Result {
    let report = crate::test_source::resolve(
        "struct S<T>(T);impl<T>S<T>{const C:u8={let n=7;n};}fn main(){}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let initializer = *facts.initializers.last().ok_or("initializer")?;
    let template =
        crate::body::templates::Prepared::default().initializer(&report, &facts, &initializer)?;
    let ty = facts
        .generics
        .parameters_in_scope(initializer.definition)
        .find_map(|parameter| match parameter.argument {
            Argument::Type(ty) => Some(ty),
            _ => None,
        })
        .ok_or("original parameter")?;
    assert!(matches!(facts.types.kind(ty), Some(Kind::Parameter(_))));
    check(&mut facts, &template.body)?;
    let instance = Instance {
        definition: initializer.definition,
        arguments: facts.types.intern_arguments(&[Argument::Type(ty)])?,
    };
    let mut body = template.body.clone();
    body.values.push(Value {
        ty: body.result,
        kind: Rvalue::AssociatedConstant(instance),
    });
    assert!(check(&mut facts, &body).is_err());
    body.values.pop();
    body.values.push(Value {
        ty,
        kind: Rvalue::Unit,
    });
    assert!(check(&mut facts, &body).is_err());
    body.values.pop();
    check(&mut facts, &body)?;
    Ok(())
}
