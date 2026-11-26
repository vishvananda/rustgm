//! Lexical declaration ancestry is validated independently of generic inheritance.
use super::{DefinitionKind, SourceRows, Work};
use crate::support::crates::generics::source::Definition;

const PRIMITIVES: [&str; 19] = [
    "bool", "char", "str", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
    "u128", "usize", "f32", "f64", "f16", "f128",
];

pub(super) fn validate(
    rows: &SourceRows,
    work: &mut Work,
) -> Result<Vec<Definition>, &'static str> {
    let count = rows.declarations.len();
    work.charge(count.saturating_mul(3))?;
    let mut ordinals = vec![0_u32; count];
    let mut colours = vec![0_u8; count];
    let mut definitions = Vec::with_capacity(count);
    let mut roots = 0;
    for row in &rows.declarations {
        work.charge(1)?;
        if row.sites == 0 || row.sites as usize > crate::support::crates::MAX_COMPONENTS {
            return Err("source declaration has no bounded diagnostic sites");
        }
        if let Some(parent) = row.definition.parent {
            let parent = parent.0 as usize;
            let actual = rows
                .declarations
                .get(parent)
                .ok_or("source lexical declaration parent is unknown")?;
            if row.definition.kind == DefinitionKind::ForeignType
                && actual.definition.kind != DefinitionKind::Module
            {
                return Err("source foreign declaration has no actual module owner");
            }
            if row.definition.kind == DefinitionKind::Primitive
                && (actual.definition.kind != DefinitionKind::Module
                    || actual.definition.parent.is_some()
                    || !row.public
                    || row
                        .name
                        .as_deref()
                        .is_none_or(|name| !PRIMITIVES.contains(&name)))
            {
                return Err(
                    "source primitive declaration changed its root owner, visibility or spelling",
                );
            }
            if !valid_child(actual.definition.kind, row) || row.ordinal != ordinals[parent] {
                return Err("source lexical child has a foreign kind, gap or duplicate ordinal");
            }
            ordinals[parent] = ordinals[parent]
                .checked_add(1)
                .ok_or("source lexical declaration ordinal overflow")?;
        } else {
            if row.definition.kind != DefinitionKind::Module || row.ordinal != 0 {
                return Err("source root is not its actual module declaration");
            }
            roots += 1;
        }
        if row.local.is_some()
            && !matches!(
                row.definition.kind,
                DefinitionKind::ValueParameter | DefinitionKind::Local
            )
            || row.definition.kind == DefinitionKind::Local && row.local.is_none()
        {
            return Err("source local declaration lost its exact body association");
        }
        if let Some(name) = &row.name {
            work.charge(name.len())?;
            if name.len() > 256 || name.chars().any(char::is_control) {
                return Err("source declaration name is outside the supported bounded spelling");
            }
        }
        definitions.push(row.definition);
    }
    if roots != 1 {
        return Err("source declarations require one genuine module root");
    }
    let mut path = Vec::with_capacity(count);
    for root in 0..count {
        work.charge(1)?;
        if colours[root] != 0 {
            continue;
        }
        let mut next = Some(root);
        while let Some(index) = next {
            work.charge(1)?;
            match colours[index] {
                2 => break,
                1 => return Err("source lexical declaration parent cycle"),
                _ => {}
            }
            colours[index] = 1;
            path.push(index);
            next = definitions[index].parent.map(|parent| parent.0 as usize);
        }
        for index in path.drain(..) {
            work.charge(1)?;
            colours[index] = 2;
        }
    }
    Ok(definitions)
}

fn valid_child(parent: DefinitionKind, child: &super::super::Declaration) -> bool {
    match parent {
        DefinitionKind::Module
        | DefinitionKind::Struct
        | DefinitionKind::Enum
        | DefinitionKind::Trait
        | DefinitionKind::SelfType
        | DefinitionKind::Function => true,
        DefinitionKind::Constant => {
            // A function-pointer type may bind lifetimes, and a closure's value
            // parameters are lexically owned by the enclosing constant. Complete
            // body/formal checks still bind their exact closure and local slot.
            match child.definition.kind {
                DefinitionKind::RegionParameter => child.local.is_none(),
                DefinitionKind::Local | DefinitionKind::ValueParameter => child.local.is_some(),
                _ => false,
            }
        }
        DefinitionKind::TypeAlias => matches!(
            child.definition.kind,
            DefinitionKind::TypeParameter | DefinitionKind::ConstParameter
        ),
        DefinitionKind::AssociatedType => matches!(
            child.definition.kind,
            DefinitionKind::TypeParameter
                | DefinitionKind::RegionParameter
                | DefinitionKind::ConstParameter
        ),
        DefinitionKind::TraitAlias => child.definition.kind == DefinitionKind::SelfType,
        _ => false,
    }
}
