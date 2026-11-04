//! Default-body policy is direct declaration data; local uses acquire no extra gate.
use crate::{
    resolve::{
        package::Error,
        unit::{NodeKey, Unit},
    },
    syntax::{
        ast::Kind,
        attribute::Attribute,
        item::{GenericFlavor, ItemKind},
    },
};

/// The pinned parser shares the stability target list, including bodyless items.
/// This does not create a body, inherit API instability or grant const permission.
pub(super) fn validate_target(
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
) -> Result<(), Error> {
    let ast = &unit.configured.ast;
    let valid = match ast.node(key.node).kind {
        Kind::Item(index) => !matches!(
            ast.items[index as usize].kind,
            ItemKind::ExternBlock { .. } | ItemKind::MacroInvocation(_)
        ),
        Kind::FieldDefinition { .. } | Kind::Variant(_) | Kind::Crate(_) => true,
        Kind::Generic(index) => {
            let parameter = ast.generics[index as usize];
            parameter.flavor == GenericFlavor::Type && parameter.default.is_some()
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(Error::at(
            key.file,
            attribute.span.start,
            "default-body stability has an unsupported attribute target",
        ))
    }
}
