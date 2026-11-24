//! Observe v14 declaration admission without forming a writable interface.
use super::{BodyTransport, CrateIdentity, Declarations, DefId, ExportMode, Report, Stage, s};
use crate::{
    resolve::{model::Definition, package::Error},
    support::bindings::LangItem,
};

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
                census.refuse(&mut stage, failure(report, id, definition, message))?;
            }
        }
        let roles = facts
            .source_bindings
            .as_deref()
            .ok_or("source role census has no completed catalog")?;
        for role in LangItem::ALL {
            stage.charge(1)?;
            let Some(declaration) = roles.language(role) else {
                continue;
            };
            if let Err(message) = stage.admitted_role(report, facts, role) {
                stage.charge(0)?;
                let id = declaration.definition;
                let mut refusal = failure(
                    report,
                    id,
                    &report.resolution.definitions[id.0 as usize],
                    message,
                );
                refusal.kind = "language-role";
                census.refuse(&mut stage, refusal)?;
            }
        }
        Ok(census)
    }
    fn refuse(&mut self, stage: &mut Stage, failure: DeclFailure) -> Result<(), &'static str> {
        let bytes = failure
            .path
            .len()
            .saturating_add(failure.diagnostic.message.len())
            .saturating_add(2 * size_of::<DeclFailure>());
        stage.reserve(bytes)?;
        self.refused.push(failure);
        Ok(())
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
