//! Unvisited parsed syntax produces a direct warning, independent of lint levels.
use super::{
    Engine, Error, NodeKey, Report, Sources, permission_frames::FrameView,
    permission_session::Query,
};
use crate::{
    resolve::{
        bindings::FeatureName,
        package::{Package, RetentionNote, Warning},
        unit::FileId,
    },
    support::{diagnostics::WarningCode, hash::HashSet},
    syntax::gate_sites::{Declarations, Kind},
};
#[path = "unvisited_syntax/sites.rs"]
mod sites;
use sites::Site;
const MESSAGE: &str = "specialization is experimental";
const ALIAS_MESSAGE: &str = "trait aliases are experimental";
#[cfg(test)]
#[path = "../../../tests/invariants/modifier_contracts.rs"]
mod tests;
enum Failure {
    Budget(Error),
    Origin(Error),
}
impl Failure {
    fn error(self) -> Error {
        match self {
            Self::Budget(error) => error.warning_retention_failed(RetentionNote::Budget),
            Self::Origin(error) => error.warning_retention_failed(RetentionNote::UnavailableOrigin),
        }
    }
    fn retain(self, primary: Error) -> Error {
        primary.warning_retention_failed(match self {
            Self::Budget(_) => RetentionNote::Budget,
            Self::Origin(_) => RetentionNote::UnavailableOrigin,
        })
    }
}
fn failure(entries: &Declarations, error: Error) -> Failure {
    if entries.exhausted() {
        Failure::Budget(error)
    } else {
        Failure::Origin(error)
    }
}
fn diagnostic(
    file: FileId,
    span: crate::syntax::token_tree::Span,
    entries: &Declarations,
    message: &str,
    project: &impl Fn(Error, &Declarations) -> Result<Error, Error>,
) -> Result<Error, Failure> {
    entries
        .charge(
            1,
            message.len()
                + size_of::<crate::resolve::package::ErrorData>()
                // Prepay coexistence with the exact terminal Vec<Warning> conversion.
                + 4 * (size_of::<(NodeKey, Warning)>() + size_of::<Warning>()),
        )
        .map_err(|message| failure(entries, Error::at(file, span.start, message)))?;
    let diagnostic = project(Error::at(file, span.start, message), entries)
        .map_err(|error| failure(entries, error))?;
    let (steps, bytes) = diagnostic.warning_payload().ok_or_else(|| {
        failure(
            entries,
            Error::at(
                file,
                span.start,
                "default warning acquired a nested carrier",
            ),
        )
    })?;
    entries
        .charge(
            steps,
            bytes
                + crate::support::retained::new_map::<
                    (
                        FileId,
                        u32,
                        Option<(std::num::NonZeroU32, std::num::NonZeroU32)>,
                    ),
                    (),
                >(1),
        )
        .map_err(|message| failure(entries, Error::at(file, span.start, message)))?;
    Ok(diagnostic)
}
fn collect(
    package: &Package,
    mut allows: impl FnMut(FileId, Site<'_>, &FeatureName<'_>) -> Result<bool, Error>,
    project: impl Fn(Error, &Declarations) -> Result<Error, Error>,
) -> Result<Vec<(NodeKey, Warning)>, Failure> {
    let mut warnings = Vec::new();
    let mut origins = None;
    for (index, unit) in package.units.iter().enumerate() {
        let Some(entries) = &unit.configured.ast.gate_sites else {
            continue;
        };
        let file = FileId(
            u32::try_from(index)
                .map_err(|_| Failure::Origin(Error::unlocated("feature warning file overflow")))?,
        );
        let items = entries.entries().iter().map(|site| Ok(Site::Item(site)));
        let expressions = entries
            .expressions()
            .iter()
            .enumerate()
            .map(|(index, site)| {
                u32::try_from(index)
                    .map(|index| Site::Expression(index, site))
                    .map_err(|_| {
                        Failure::Origin(Error::unlocated("expression-gate ordinal overflow"))
                    })
            });
        for site in items.chain(expressions) {
            let site = site?;
            let Some(message) = sites::prepare(package, file, unit, entries, site, &mut allows)?
            else {
                continue;
            };
            let diagnostic = diagnostic(file, site.span(), entries, message, &project)?;
            let origin = diagnostic.file.source().ok_or_else(|| {
                failure(
                    entries,
                    Error::at(file, site.span().start, "feature warning lost its source"),
                )
            })?;
            if !origins.get_or_insert_with(HashSet::default).insert((
                origin,
                diagnostic.span.start,
                site.context(),
            )) {
                continue;
            }
            warnings.push((
                NodeKey {
                    file,
                    node: site.node(),
                },
                Warning {
                    code: WarningCode::UnstableSyntaxPreExpansion,
                    diagnostic,
                },
            ));
        }
    }
    Ok(warnings)
}
fn join(primary: Error, warnings: Result<Vec<(NodeKey, Warning)>, Failure>) -> Error {
    match warnings {
        Ok(warnings) => {
            // Every retained source row explicitly prepaid both vectors above.
            let mut retained = Vec::with_capacity(warnings.len());
            for (_, warning) in warnings {
                retained.push(warning);
            }
            primary.retain_warnings(retained)
        }
        Err(failure) => failure.retain(primary),
    }
}
impl Report {
    fn default_warnings(&self) -> Result<Vec<(NodeKey, Warning)>, Failure> {
        let mut session = self.macro_allowances();
        collect(
            &self.package,
            |file, site, feature| match site {
                Site::Item(site) => session.allows_at(file, site.span, feature),
                Site::Expression(ordinal, site) => {
                    session.allows_try_block(file, ordinal, *site, feature)
                }
            },
            |error, entries| {
                self.diagnostic_charged(error, |steps, bytes| entries.charge(steps, bytes))
            },
        )
    }
    pub(crate) fn pre_expansion_warnings(&self) -> Result<Vec<(NodeKey, Warning)>, Error> {
        self.default_warnings().map_err(Failure::error)
    }
    pub(crate) fn retain_expansion_warnings(&self, primary: Error) -> Error {
        join(
            self.retain_visibility_warnings(primary),
            self.default_warnings(),
        )
    }
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn retain_expansion_warnings(&self, primary: Error) -> Error {
        let mut memo = None;
        let mut query = Query {
            view: FrameView {
                package: &self.loader.package,
                definitions: &self.definitions,
                invocations: &self.invocations,
                facts: &self.paths.state.facts,
            },
            memo: &mut memo,
        };
        let warnings = collect(
            &self.loader.package,
            |file, site, feature| match site {
                Site::Item(site) => query.allows(
                    super::Location {
                        file,
                        span: site.span,
                    },
                    feature,
                ),
                Site::Expression(_, site) => query.allows_composed(
                    super::Location {
                        file,
                        span: site.keyword,
                    },
                    super::Location {
                        file,
                        span: site.construct,
                    },
                    feature,
                ),
            },
            |error, entries| {
                self.diagnostic_charged(error, |steps, bytes| entries.charge(steps, bytes))
            },
        );
        join(self.retain_visibility_warnings(primary), warnings)
    }
}
