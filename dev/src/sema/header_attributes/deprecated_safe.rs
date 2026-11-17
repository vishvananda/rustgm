//! Keep the edition migration note attached to its actual function header.
use super::{Declarations, DefId, Error, Header, NodeKey, Report, ledger};
use crate::{
    lex::token::Punctuation,
    resolve::{bindings::FeatureName, expansion::MacroAllowanceSession, model::DefKind},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};

pub(super) struct Marker {
    owner_slot: usize,
    effect: usize,
    attribute: Attribute,
    audit: Box<str>,
}

pub(in crate::sema) fn consume(
    report: &Report,
    facts: &mut Declarations,
    (owner, key): (NodeKey, NodeKey),
    (attribute, effect): (Attribute, usize),
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(key, attribute.span)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("deprecated-safe attribute lost its name"))?
        .span;
    if report.package.source_core().is_none()
        || (!report.package.requested_internal_attributes()
            && !allowances.allows_at(
                key.file,
                name,
                &FeatureName::new("rustc_attrs").map_err(error)?,
            )?)
    {
        return Err(error(
            "rustc_deprecated_safe_2024 requires the rustc_attrs source profile",
        ));
    }
    let unit = report.package.unit(key.file);
    let ast = &unit.configured.ast;
    if !matches!(ast.node(key.node).kind, Kind::Item(index)
        if matches!(ast.items[index as usize].kind, ItemKind::Function { .. }))
    {
        return Err(error(
            "rustc_deprecated_safe_2024 requires a function or method",
        ));
    }
    let actual = header(facts, key)
        .ok_or_else(|| error("deprecated-safe declaration has no retained header"))?;
    super::charge_source_contract_query(facts, actual.owners.len())?;
    let owner_slot = actual
        .owners
        .iter()
        .position(|row| row.key == owner && row.effects.contains(&effect))
        .ok_or_else(|| error("deprecated-safe effect belongs to another owner"))?;
    if marker(actual).is_some() {
        return Err(error("duplicate rustc_deprecated_safe_2024 attribute"));
    }
    let promises = usize::from(actual.promises.is_none()) * size_of::<super::Promises>();
    let source = unit.source.text();
    let token = literal(source, attribute, forest).map_err(error)?;
    let bytes = (token.end - token.start) as usize;
    super::reserve_source_contracts(
        facts,
        bytes.saturating_add(1),
        bytes
            .saturating_add(promises)
            .saturating_add(size_of::<Marker>()),
    )?;
    let Some(crate::literal::Value::String(decoded)) =
        crate::literal::interpret(source, token).map_err(|e| error(e.message))?
    else {
        return Err(error(
            "rustc_deprecated_safe_2024 audit_that requires a string literal",
        ));
    };
    let audit = String::from_utf8(decoded.into_owned())
        .map_err(|_| error("invalid UTF-8 in rustc_deprecated_safe_2024 audit_that"))?;
    let headers = &mut ledger(facts, 0)?.local;
    headers.first_deprecated_safe_2024.get_or_insert(key);
    headers
        .declarations
        .get_mut(&key)
        .ok_or_else(|| error("deprecated-safe declaration header disappeared"))?
        .promises
        .get_or_insert_default()
        .deprecated_safe_2024 = Some(Box::new(Marker {
        owner_slot,
        effect,
        attribute,
        audit: audit.into_boxed_str(),
    }));
    Ok(())
}

fn literal(
    source: &str,
    attribute: Attribute,
    forest: &Forest,
) -> Result<crate::lex::token::Token, &'static str> {
    let Input::Delimited(group) = attribute.input else {
        return Err("rustc_deprecated_safe_2024 requires one audit_that argument");
    };
    if attribute.unsafe_wrapper {
        return Err("rustc_deprecated_safe_2024 is not an unsafe attribute");
    }
    let range = forest
        .children(group)
        .ok_or("invalid rustc_deprecated_safe_2024 input")?;
    let start = range.start;
    if !(range.end - start == 3
        || range.end - start == 4 && forest.punctuation(start + 3, Punctuation::Comma))
        || forest.name(source, start) != Some("audit_that")
        || !forest.punctuation(start + 1, Punctuation::Equal)
    {
        return Err("rustc_deprecated_safe_2024 requires one audit_that string assignment");
    }
    forest
        .token(start + 2)
        .ok_or("rustc_deprecated_safe_2024 audit_that requires a string literal")
}

fn headers(facts: &Declarations) -> Option<&super::Headers> {
    Some(
        &facts
            .must_use_contracts
            .as_deref()?
            .headers
            .as_deref()?
            .local,
    )
}

fn header(facts: &Declarations, key: NodeKey) -> Option<&Header> {
    headers(facts)?.declarations.get(&key)
}

fn marker(header: &Header) -> Option<&Marker> {
    header.promises.as_deref()?.deprecated_safe_2024.as_deref()
}

pub(super) fn bind(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    let Some(mark) = header(facts, key).and_then(marker) else {
        return Ok(());
    };
    super::charge_source_contract_query(facts, 2)?;
    let actual = &report.resolution.definitions[definition.0 as usize];
    if actual.node != Some(key)
        || actual.kind != DefKind::Function
        || report.resolution.foreign_owners.contains_key(&definition)
    {
        return Err(Error::at(
            key.file,
            mark.attribute.span.start,
            "rustc_deprecated_safe_2024 requires a nonforeign function or method",
        ));
    }
    Ok(())
}

impl Declarations {
    pub(in crate::sema) fn has_deprecated_safe_2024(&self) -> bool {
        headers(self).is_some_and(|headers| headers.first_deprecated_safe_2024.is_some())
    }

    pub(in crate::sema) fn is_deprecated_safe_2024(&self, definition: DefId) -> bool {
        headers(self)
            .filter(|headers| headers.first_deprecated_safe_2024.is_some())
            .and_then(|headers| {
                headers
                    .definitions
                    .get(&definition)
                    .and_then(|key| headers.declarations.get(key))
            })
            .and_then(marker)
            .is_some()
    }

    pub(in crate::sema) fn deprecated_safe_2024_audit(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<Option<&str>, Error> {
        let Some(headers) = headers(self) else {
            return Ok(None);
        };
        let Some(&key) = headers.definitions.get(&definition) else {
            return Ok(None);
        };
        let Some(actual) = headers.declarations.get(&key) else {
            return Ok(None);
        };
        let Some(mark) = marker(actual) else {
            return Ok(None);
        };
        bind(report, self, definition, key)?;
        super::validate_source(self, report, key, actual)?;
        let owner = actual
            .owners
            .get(mark.owner_slot)
            .ok_or_else(|| Error::unlocated("deprecated-safe owner slot changed"))?;
        let unit = report.package.unit(key.file);
        if !owner.effects.contains(&mark.effect)
            || !unit
                .retained_attributes(mark.effect..mark.effect + 1)
                .any(|(attribute, forest)| {
                    mark.attribute == attribute
                        && attribute.name(forest, unit.source.text())
                            == Some("rustc_deprecated_safe_2024")
                })
        {
            return Err(Error::unlocated(
                "deprecated-safe attribute changed its source effect",
            ));
        }
        Ok(Some(&mark.audit))
    }

    pub(crate) fn require_deprecated_safe_transport(&self) -> Result<(), Error> {
        let Some(key) = headers(self).and_then(|headers| headers.first_deprecated_safe_2024) else {
            return Ok(());
        };
        let mark = header(self, key)
            .and_then(marker)
            .ok_or_else(|| Error::unlocated("deprecated-safe publication lost its marker"))?;
        Err(Error::at(
            key.file,
            mark.attribute.span.start,
            "rustc_deprecated_safe_2024 requires declaration and call-origin transport",
        ))
    }
}
