//! Observe v14 declaration admission without forming a writable interface.
use super::{BodyTransport, CrateIdentity, Declarations, DefId, ExportMode, Report, Stage, s};
use crate::resolve::{model::Definition, package::Error};

/// The real v14 declaration and nominal shape gates that need no exported rows.
pub struct DeclCensus {
    pub checked: usize,
    pub refused: Vec<DeclFailure>,
}
pub struct DeclFailure {
    pub declaration: u32,
    pub kind: &'static str,
    pub path: String,
    pub diagnostic: Error,
}
impl DeclCensus {
    pub(crate) fn collect(
        report: &Report,
        facts: &Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        let mode = ExportMode::CompilerOperations;
        let mut stage = Stage::header_draft(report, facts, identity, mode, BodyTransport::Nodes)?;
        let mut census = Self {
            checked: 0,
            refused: Vec::new(),
        };
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            stage.charge(1)?;
            let id =
                DefId(u32::try_from(index).map_err(|_| "source declaration identity overflow")?);
            let result = stage
                .exported_kind(report, facts, id, mode)
                .and_then(|kind| match kind {
                    Some(s::DefinitionKind::Struct | s::DefinitionKind::Enum) => {
                        stage.charge(1)?;
                        stage.nominal_shape(report, facts, id).map(drop)
                    }
                    _ => Ok(()),
                });
            census.checked += 1;
            if let Err(message) = result {
                // The same work ledger remains sticky after a failed admission.
                stage.charge(0)?;
                let failure = failure(report, id, definition, message);
                let bytes = failure
                    .path
                    .len()
                    .saturating_add(message.len())
                    .saturating_add(2 * size_of::<DeclFailure>());
                stage.reserve(bytes)?;
                census.refused.push(failure);
            }
        }
        Ok(census)
    }
}

fn failure(
    report: &Report,
    id: DefId,
    definition: &Definition,
    message: &'static str,
) -> DeclFailure {
    let diagnostic = definition.node.map_or_else(
        || Error::unlocated(message),
        |key| report.diagnostic(Declarations::error(report, key, message)),
    );
    let package = &report.package;
    let mut module = Some(report.resolution.scopes[definition.scope.0 as usize].module);
    let mut names = vec![package.names.get(definition.name)];
    while let Some(index) = module {
        let current = &package.modules[index.0 as usize];
        names.push(package.names.get(current.name));
        module = current.parent;
    }
    names.reverse();
    DeclFailure {
        declaration: id.0,
        kind: definition.kind.name(),
        path: names.join("::"),
        diagnostic,
    }
}
