//! A declaration is reserved from its actual typed owner and lexical scope.
use super::{
    DefId, DefKind, Definition, Error, Expected, Imported, Interface, ModuleId, Namespace,
    Resolver, ScopeId, Work, error, g,
};
use crate::resolve::model::Access;
type Owners = (Vec<DefId>, Vec<Expected>, Vec<Option<ScopeId>>);
pub(super) fn functions(
    resolver: &mut Resolver<'_>,
    scope: ScopeId,
    artifact: &Interface,
    imported: &mut Imported,
    work: &mut Work,
) -> Result<Owners, Error> {
    let rows = artifact
        .generic_payload()
        .ok_or_else(|| error("missing generic declaration rows"))?
        .rows();
    let mut definitions = Vec::with_capacity(rows.declarations.len());
    let mut expected = Vec::with_capacity(rows.declarations.len());
    let mut owners = vec![None; rows.declarations.len()];
    // Named functions precede parameters and locals in the validated table.
    for declaration in &rows.declarations {
        work.charge(1)?;
        let g::Declaration::Function { identity, public } = declaration else {
            break;
        };
        let definition = if *public && artifact.public_names().is_none() {
            resolver.import_item(
                scope,
                identity,
                DefKind::Function,
                &mut imported.definitions,
            )?
        } else {
            resolver.import_unbound_item(
                scope,
                identity,
                DefKind::Function,
                if *public {
                    Access::Public
                } else {
                    Access::ExternalPrivate
                },
                &mut imported.definitions,
            )?
        };
        let actual = &resolver.facts.definitions[definition.0 as usize];
        expected.push(Expected {
            scope: actual.scope,
            kind: DefKind::Function,
            namespace: Namespace::Value,
            name: actual.name,
        });
        owners[definitions.len()] =
            Some(resolver.new_scope(Some(actual.scope), ModuleId(0), true)?);
        definitions.push(definition);
    }
    Ok((definitions, expected, owners))
}
pub(super) fn bindings(
    resolver: &mut Resolver<'_>,
    rows: &g::Rows,
    owners: &[Option<ScopeId>],
    locations: &[Option<ScopeId>],
    definitions: &mut Vec<DefId>,
    expected: &mut Vec<Expected>,
    work: &mut Work,
) -> Result<(), Error> {
    for (ordinal, declaration) in rows.declarations.iter().enumerate().skip(definitions.len()) {
        work.charge(1)?;
        let (scope, kind, namespace, name) = match declaration {
            g::Declaration::TypeParameter { owner, name, .. } => (
                owners[owner.0 as usize]
                    .ok_or_else(|| error("missing imported parameter owner"))?,
                DefKind::Generic,
                Namespace::Type,
                name,
            ),
            g::Declaration::ValueParameter { name, .. } | g::Declaration::Local { name, .. } => (
                locations[ordinal].ok_or_else(|| error("missing actual imported local scope"))?,
                if matches!(declaration, g::Declaration::ValueParameter { .. }) {
                    DefKind::Parameter
                } else {
                    DefKind::Local
                },
                Namespace::Value,
                name,
            ),
            g::Declaration::Function { .. } => {
                return Err(error("reordered imported function owner"));
            }
        };
        let name = resolver.package.names.intern(name).map_err(error)?;
        let definition = DefId(
            u32::try_from(resolver.facts.definitions.len())
                .map_err(|_| error("too many imported local declarations"))?,
        );
        resolver.facts.definitions.push(Definition {
            name,
            namespace,
            kind,
            scope,
            node: None,
            access: Access::ExternalPrivate,
            members: None,
        });
        expected.push(Expected {
            scope,
            kind,
            namespace,
            name,
        });
        definitions.push(definition);
    }
    Ok(())
}
