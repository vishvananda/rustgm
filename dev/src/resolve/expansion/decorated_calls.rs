//! Active macro attributes keep their actual configured owner and parsed path.
#[cfg(test)]
#[path = "../../../tests/invariants/decorated_owners.rs"]
mod tests;
use super::{Call, CallGrammar, Engine, Error, FileId, Package, Sources, State, Visit};
use crate::{
    expand::configured::Effect,
    lex::token::Punctuation,
    resolve::paths::{Buffer, Segments},
    support::decorations::{Consumer, PredefinedTool},
    syntax::attribute::Input,
};

impl<S: Sources> Engine<'_, S> {
    /// Compiler attributes retain their owning validators. The first real macro
    /// attribute blocks its owner until lookup completes; no child is evaluated.
    pub(super) fn attribute_call(&mut self, visit: Visit, file: FileId) -> Result<bool, Error> {
        let selected = self.macro_attribute(visit, file)?;
        let Some((effect, offset)) = selected else {
            return Ok(false);
        };
        if self.calls.len() >= 1_000_000 {
            return Err(Error::at(
                file,
                offset,
                "crate attribute work budget exceeded",
            ));
        }
        charge(
            &self.loader.package,
            file,
            offset,
            1,
            4 * (size_of::<Call>() + size_of::<usize>()),
        )?;
        let (absolute, path) = attribute_path(&mut self.loader.package, file, effect)?;
        if self
            .paths
            .state
            .facts
            .scopes
            .get(visit.path_scope.0 as usize)
            .is_none_or(|scope| scope.module != visit.module)
            || self
                .paths
                .state
                .facts
                .node_scopes
                .get(&(visit.module, visit.node))
                .is_some_and(|scope| *scope != visit.path_scope)
        {
            return Err(Error::at(
                file,
                offset,
                "attribute use has a different actual caller scope",
            ));
        }
        let index = self.calls.len();
        // Attribute uses store the exact configured-effect index. Checked Bang
        // extraction prevents interpreting it as a bang argument capture index.
        let capture = u32::try_from(effect)
            .map_err(|_| Error::at(file, offset, "too many configured attribute uses"))?;
        // The ordinary source scope index binds this real attributed wrapper;
        // publication never substitutes its later underlying item/body scope.
        self.paths
            .state
            .facts
            .node_scopes
            .insert(visit.module, file, visit.node, visit.path_scope);
        self.calls.push(Call {
            visit,
            file,
            capture,
            grammar: CallGrammar::Attribute,
            absolute,
            path: path.save(),
            scope: self.head,
            output: None,
            state: State::Unresolved,
            queued: true,
            selected: None,
            error: None,
            offset,
            eager: None,
            notified: false,
        });
        self.ready.push_back(index);
        Ok(true)
    }
    fn macro_attribute(&self, visit: Visit, file: FileId) -> Result<Option<(usize, u32)>, Error> {
        let unit = self.loader.package.unit(file);
        let owner_index = unit.owner_indices[visit.node.get() as usize]
            .ok_or_else(|| Error::at(file, 0, "attribute use has no configured owner"))?;
        let owner = &unit.configured.owners[owner_index.get() as usize - 1];
        if owner.node != visit.node || !owner.enabled {
            return Err(Error::at(
                file,
                0,
                "attribute use has a different configured owner",
            ));
        }
        for index in owner.attributes.clone() {
            charge(&self.loader.package, file, 0, 1, 0)?;
            let effect = &unit.configured.effects[index];
            if effect.attribute.effect != Effect::Keep {
                continue;
            }
            let attribute = effect.attribute.attribute;
            let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
            let name = attribute.name(forest, unit.source.text());
            charge(
                &self.loader.package,
                file,
                attribute.span.start,
                name.map_or(0, str::len),
                0,
            )?;
            if name == Some("register_tool") {
                return Err(Error::at(
                    file,
                    attribute.span.start,
                    "source tool registration is not implemented",
                ));
            }
            if matches!(attribute.input, Input::Doc(_)) || name.and_then(Consumer::named).is_some()
            {
                continue;
            }
            let path = attribute.path;
            if path.end == path.start + 3
                && forest.punctuation(path.start + 1, Punctuation::ColonColon)
                && forest.name(unit.source.text(), path.start).zip(forest.name(unit.source.text(), path.start + 2))
                    .and_then(|(tool, name)| Consumer::tool_member(tool, name)).is_some()
            {
                continue;
            }
            if path.end.saturating_sub(path.start) >= 3
                && forest.punctuation(path.start + 1, Punctuation::ColonColon)
                && forest
                    .name(unit.source.text(), path.start)
                    .and_then(PredefinedTool::named)
                    .is_some()
            {
                continue;
            }
            return Ok(Some((index, attribute.span.start)));
        }
        Ok(None)
    }
}

pub(super) fn validate(
    package: &Package,
    facts: &crate::resolve::model::Resolution,
    call: &Call,
) -> Result<(), Error> {
    let error = |message| call.diagnostic(message);
    charge(package, call.file, call.offset, 1, 0)?;
    if package
        .modules
        .get(call.visit.module.0 as usize)
        .is_none_or(|module| module.file != call.file)
        || facts
            .scopes
            .get(call.visit.path_scope.0 as usize)
            .is_none_or(|scope| scope.module != call.visit.module)
        || facts.node_scopes.get(&(call.visit.module, call.visit.node))
            != Some(&call.visit.path_scope)
    {
        return Err(error("attribute call actual caller scope changed"));
    }
    let unit = package.unit(call.file);
    let owner = unit
        .owner_indices
        .get(call.visit.node.get() as usize)
        .copied()
        .flatten()
        .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
        .ok_or_else(|| error("attribute call lost its actual owner"))?;
    let index = call.capture as usize;
    let effect = unit
        .configured
        .effects
        .get(index)
        .ok_or_else(|| error("attribute call lost its actual configured effect"))?;
    let active = call.state != State::Removed;
    if owner.enabled != active
        || owner.node != call.visit.node
        || !owner.attributes.contains(&index)
        || effect.attribute.effect != Effect::Keep
        || effect.attribute.attribute.span.start != call.offset
        || unit.configured.active.get(call.visit.node.get() as usize) != Some(&active)
    {
        return Err(error("attribute call configured identity changed"));
    }
    Ok(())
}

fn attribute_path(
    package: &mut Package,
    file: FileId,
    index: usize,
) -> Result<(bool, Buffer), Error> {
    let unit = &package.units[file.0 as usize];
    let effect = &unit.configured.effects[index];
    let attribute = effect.attribute.attribute;
    let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
    let range = attribute.path;
    let offset = attribute.span.start;
    let count = range
        .end
        .checked_sub(range.start)
        .filter(|count| *count != 0 && range.end as usize <= forest.nodes().len())
        .ok_or_else(|| Error::at(file, offset, "attribute path has an invalid retained range"))?;
    charge(
        package,
        file,
        offset,
        count as usize,
        4 * count as usize
            * (size_of::<crate::support::names::NameId>()
                + size_of::<crate::expand::marks::ItemContext>()),
    )?;
    let absolute = forest.punctuation(range.start, Punctuation::ColonColon);
    let mut cursor = range.start + u32::from(absolute);
    let mut path = Buffer::default();
    while cursor < range.end {
        let tree = forest
            .tree(cursor)
            .ok_or_else(|| Error::at(file, offset, "attribute path token is missing"))?;
        let name = forest.name(unit.source.text(), cursor).ok_or_else(|| {
            Error::at(
                file,
                tree.span.start,
                "attribute path requires an actual identifier",
            )
        })?;
        charge(package, file, tree.span.start, name.len(), 4 * name.len())?;
        let name = package
            .names
            .intern(name)
            .map_err(|message| Error::at(file, tree.span.start, message))?;
        path.append(package, file, tree.span, name)?;
        cursor += 1;
        if cursor < range.end {
            if !forest.punctuation(cursor, Punctuation::ColonColon) || cursor + 1 == range.end {
                return Err(Error::at(
                    file,
                    offset,
                    "attribute path has invalid retained punctuation",
                ));
            }
            cursor += 1;
        }
    }
    if path.is_empty() {
        return Err(Error::at(
            file,
            offset,
            "attribute path has no actual identifiers",
        ));
    }
    Ok((absolute, path))
}

fn charge(
    package: &Package,
    file: FileId,
    offset: u32,
    steps: usize,
    bytes: usize,
) -> Result<(), Error> {
    if let Some(source) = package.source_core() {
        source
            .macros
            .charge(steps, bytes)
            .map_err(|message| Error::at(file, offset, message))?;
    }
    Ok(())
}
