//! Parsed type segments retain their original namespace before a value suffix.
use super::{Context, DefKind, Error, Lookup, LookupMode, Namespace, NodeId, NodeKey, Resolver};
use crate::resolve::{
    model::{AssociatedPath, QualifiedPath},
    paths::Identified,
};

impl Resolver<'_> {
    pub(super) fn projected_value_path(
        &mut self,
        c: Context,
        node: NodeId,
        (absolute, names): (bool, Identified<'_>),
        mut path: AssociatedPath,
    ) -> Result<AssociatedPath, Error> {
        if path.namespace != Namespace::Value
            || (path.suffix.len() == 1
                && !(self.facts.definitions[path.owner.0 as usize].kind == DefKind::TypeAlias
                    && self.facts.scopes
                        [self.facts.definitions[path.owner.0 as usize].scope.0 as usize]
                        .associated))
        {
            return Ok(path);
        }
        let length = names
            .len()
            .checked_sub(1)
            .ok_or_else(|| self.error(c, node, "missing projected type prefix"))?;
        let found = self.lookup_path(
            c,
            node,
            absolute,
            names.prefix(length),
            LookupMode::Use(Namespace::Type),
            None,
        )?;
        let (owner, start) = match found {
            Lookup::Associated(owner, start) | Lookup::Member(_, owner, start)
                if matches!(
                    self.facts.definitions[owner.0 as usize].kind,
                    DefKind::Generic | DefKind::SelfType
                ) =>
            {
                (owner, start)
            }
            _ => return Ok(path),
        };
        let Some((_, segments)) = self.package.unit(c.file).configured.ast.path_segments(node)
        else {
            return Err(self.error(c, node, "projected value requires parsed path segments"));
        };
        let prefix = *segments
            .get(length - 1)
            .ok_or_else(|| self.error(c, node, "missing parsed projected prefix"))?;
        let base = start
            .checked_sub(1)
            .and_then(|index| segments.get(index))
            .copied()
            .ok_or_else(|| self.error(c, node, "missing parsed projected owner"))?;
        let key = NodeKey {
            file: c.file,
            node: prefix,
        };
        self.facts.associated_paths.insert(
            key,
            AssociatedPath {
                owner,
                namespace: Namespace::Type,
                scope: c.scope,
                prefix: base,
                suffix: segments[start..length]
                    .iter()
                    .copied()
                    .zip(names[start..length].iter().copied())
                    .collect(),
            },
        );
        path.owner = owner;
        path.prefix = prefix;
        let last = path
            .suffix
            .pop()
            .ok_or_else(|| self.error(c, node, "missing projected value suffix"))?;
        path.suffix.clear();
        path.suffix.push(last);
        Ok(path)
    }

    pub(super) fn projected_qualified_path(
        &mut self,
        c: Context,
        node: NodeId,
        mut path: QualifiedPath,
    ) -> Result<QualifiedPath, Error> {
        if path.namespace != Namespace::Value || path.suffix.len() < 2 {
            return Ok(path);
        }
        let last = path
            .suffix
            .pop()
            .ok_or_else(|| self.error(c, node, "missing qualified value suffix"))?;
        let prefix = path
            .suffix
            .last()
            .copied()
            .ok_or_else(|| self.error(c, node, "missing qualified type prefix"))?;
        self.facts.qualified_paths.insert(
            NodeKey {
                file: c.file,
                node: prefix.0,
            },
            QualifiedPath {
                namespace: Namespace::Type,
                scope: path.scope,
                ty: path.ty,
                trait_path: path.trait_path,
                suffix: std::mem::take(&mut path.suffix),
            },
        );
        path.ty = prefix.0;
        path.trait_path = None;
        path.suffix.push(last);
        Ok(path)
    }
}
