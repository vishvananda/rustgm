//! Indexed source causes share one warning consumer at success and error joins.
mod recorded_owners;
use super::{Engine, Error, NodeKey, Report, Sources, Visit, attribute_effects::Effects};
use crate::resolve::{
    lint_policy::{
        Level,
        producer::{Producer, Tail},
    },
    model::Resolution,
    package::{Package, RetentionNote, Warning},
};
use crate::support::{diagnostics::WarningCode, hash::HashSet};
use crate::syntax::item::{Item, ItemKind, Visibility};
use recorded_owners::Record;

const MESSAGE: &str = "visibility qualifiers have no effect on const _ declarations";
#[cfg(test)]
#[path = "../../../tests/invariants/unnameable_items.rs"]
mod tests;
#[derive(Default)]
pub(super) struct Causes {
    records: Vec<Record>,
}
enum Failure {
    Budget(Error),
    Origin(Error),
}
impl Failure {
    fn error(self) -> Error {
        match self {
            Self::Budget(error) | Self::Origin(error) => error,
        }
    }
    fn preserve(self, primary: Error) -> Error {
        primary.warning_retention_failed(match self {
            Self::Budget(_) => RetentionNote::Budget,
            Self::Origin(_) => RetentionNote::UnavailableOrigin,
        })
    }
}
fn classify(producer: &Producer, error: Error) -> Failure {
    if producer.diagnostic_budget_exhausted() {
        Failure::Budget(error)
    } else {
        Failure::Origin(error)
    }
}
fn collect(
    effects: Option<&Effects>,
    package: &Package,
    resolution: &Resolution,
    project: impl Fn(Error) -> Error,
) -> Result<Vec<(NodeKey, Warning)>, Failure> {
    let Some(effects) = effects else {
        return Ok(Vec::new());
    };
    let Some(causes) = &effects.visibilities else {
        return Ok(Vec::new());
    };
    let missing = || {
        Failure::Origin(Error::unlocated(
            "visibility causes lost their lint producer",
        ))
    };
    let producer = effects.policy.as_ref().ok_or_else(missing)?;
    let mut warnings = Vec::new();
    let mut validated = HashSet::default();
    let mut origins = HashSet::default();
    for record in &causes.records {
        producer
            .diagnostic_read(record.key, 0)
            .map_err(|error| Failure::Budget(error.into()))?;
        producer
            .validate_diagnostic_path(package, record.tail, record.key, &mut validated)
            .map_err(|error| classify(producer, error.into()))?;
        record
            .validate(package, resolution, producer)
            .map_err(|error| classify(producer, error))?;
        producer
            .diagnostic_read(
                record.key,
                MESSAGE.len()
                    + size_of::<crate::resolve::package::ErrorData>()
                    + 4 * size_of::<(NodeKey, Warning)>(),
            )
            .map_err(|error| Failure::Budget(error.into()))?;
        let diagnostic = project(Error::at(record.key.file, record.offset(), MESSAGE));
        let (steps, bytes) = diagnostic.warning_payload().ok_or_else(|| {
            Failure::Origin(Error::at(
                record.key.file,
                record.offset(),
                "visibility warning acquired a nested carrier",
            ))
        })?;
        producer
            .diagnostic_payload(record.key, steps, bytes)
            .map_err(|error| Failure::Budget(error.into()))?;
        let file = diagnostic.file.source().ok_or_else(|| {
            Failure::Origin(Error::at(
                record.key.file,
                record.offset(),
                "source visibility warning lost its physical origin",
            ))
        })?;
        producer
            .diagnostic_read(
                record.key,
                4 * size_of::<(
                    crate::resolve::unit::FileId,
                    u32,
                    crate::expand::marks::SyntaxContextId,
                )>(),
            )
            .map_err(|error| Failure::Budget(error.into()))?;
        if !origins.insert((file, diagnostic.span.start, record.context)) {
            continue;
        }
        warnings.push((
            record.key,
            Warning {
                code: WarningCode::UnusedVisibilities,
                diagnostic,
            },
        ));
    }
    Ok(warnings)
}
fn join(primary: Error, warnings: Result<Vec<(NodeKey, Warning)>, Failure>) -> Error {
    match warnings {
        Ok(warnings) => {
            primary.retain_warnings(warnings.into_iter().map(|(_, warning)| warning).collect())
        }
        Err(failure) => failure.preserve(primary),
    }
}
impl Report {
    pub(crate) fn unused_visibility_warnings(&self) -> Result<Vec<(NodeKey, Warning)>, Error> {
        collect(
            self.attribute_effects.as_deref(),
            &self.package,
            &self.resolution,
            |error| self.diagnostic(error),
        )
        .map_err(Failure::error)
    }
    pub(crate) fn retain_visibility_warnings(&self, primary: Error) -> Error {
        join(
            primary,
            collect(
                self.attribute_effects.as_deref(),
                &self.package,
                &self.resolution,
                |error| self.diagnostic(error),
            ),
        )
    }
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn retain_visibility_warnings(&self, primary: Error) -> Error {
        join(
            primary,
            collect(
                self.attribute_effects.as_deref(),
                &self.loader.package,
                &self.paths.state.facts,
                |error| self.diagnostic(error),
            ),
        )
    }
    pub(super) fn unnamed_visibility(
        &mut self,
        visit: Visit,
        item: Item,
        tail: Option<Tail>,
    ) -> Result<(), Error> {
        if !matches!(item.kind, ItemKind::Const { .. }) || item.visibility == Visibility::Private {
            return Ok(());
        }
        let key = self.visit_key(visit);
        let unit = self.loader.package.unit(key.file);
        if item
            .name
            .is_none_or(|name| unit.spelling(name).ok() != Some("_"))
        {
            return Ok(());
        }
        let level = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
            .map_or(Ok(Level::Warn), |producer| {
                producer
                    .policy(&self.loader.package, tail, key)
                    .map(|policy| policy.unused_visibilities)
                    .map_err(Error::from)
            })?;
        if level == Level::Allow {
            return Ok(());
        }
        if level.denies() {
            return Err(Error::at(
                key.file,
                unit.configured.ast.node(key.node).span.start,
                MESSAGE,
            ));
        }
        self.reserve_visibility_cause(key)?;
        let record = Record::capture(
            &self.loader.package,
            &self.paths.state.facts,
            visit,
            item,
            tail,
        )?;
        self.attribute_effects
            .get_or_insert_default()
            .visibilities
            .get_or_insert_default()
            .records
            .push(record);
        Ok(())
    }
    fn reserve_visibility_cause(&mut self, key: NodeKey) -> Result<(), Error> {
        let frames = self.frame_mark()?;
        let calls = u32::try_from(self.calls.len())
            .map_err(|_| Error::at(key.file, 0, "visibility lint call index overflow"))?;
        let bytes = 4 * size_of::<Record>() + size_of::<Causes>() + size_of::<Effects>();
        if self.attribute_effects.is_none() {
            let mut producer = Producer::new(frames, calls);
            producer.reserve_diagnostic(key, bytes)?;
            self.attribute_effects = Some(Box::new(Effects {
                policy: Some(producer),
                ..Effects::default()
            }));
        } else {
            self.attribute_effects
                .as_deref_mut()
                .ok_or_else(|| Error::at(key.file, 0, "visibility owner disappeared"))?
                .policy
                .get_or_insert_with(|| Producer::new(frames, calls))
                .reserve_diagnostic(key, bytes)?;
        }
        Ok(())
    }
}
