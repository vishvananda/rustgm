//! Dense catalog rows own exact remaps; no row is a checked body or permission.
use super::{
    BodyOrigin, DeclarationBinding, Edition, Epoch, ImportedBodyId, ImportedDeclarationId,
    ImportedOwnerId, Label, NEXT_CATALOG, ProviderFile,
};
use crate::{
    resolve::model::{DefId, Definition},
    support::{
        crates::{
            CrateIdentity,
            owner_catalog::{BodyCatalog, BodyKind, BodyRef},
        },
        hash::HashMap,
    },
};
use std::{cell::Cell, ops::Range, rc::Rc};

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub bytes: usize,
    pub steps: usize,
    pub bodies: usize,
    pub artifacts: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            bytes: crate::support::crates::MAX_BYTES,
            steps: crate::support::crates::MAX_COMPONENTS,
            bodies: BodyCatalog::MAX_BODIES,
            artifacts: crate::support::crates::MAX_DEPENDENCIES,
        }
    }
}
pub(crate) struct Binding {
    pub unit: BodyRef,
    pub sites: Range<u32>,
}
pub(crate) struct Input {
    pub owners: BodyCatalog,
    pub edition: Edition,
    pub definitions: Vec<DefId>,
    pub declarations: Vec<DeclarationBinding>,
    pub bindings: Vec<Binding>,
    pub files: Vec<ProviderFile>,
    pub labels: Vec<Option<Label>>,
    pub declaration_labels: Vec<Option<Label>>,
}
pub(super) struct Artifact {
    pub owner: CrateIdentity,
    pub edition: Edition,
    pub definitions: Vec<DefId>,
    pub files: Vec<ProviderFile>,
    pub labels: Vec<Option<Label>>,
    pub declaration_labels: Vec<Option<Label>>,
}
pub(crate) struct Row {
    pub definition: DefId,
    pub kind: BodyKind,
    pub parent: Option<ImportedBodyId>,
    pub unit: ImportedBodyId,
    pub(super) artifact: u32,
    pub(super) body: BodyRef,
    pub(super) sites: Range<u32>,
}
pub(super) struct DeclarationRow {
    pub definition: DefId,
    pub kind: crate::resolve::model::DefKind,
    pub artifact: u32,
    pub ordinal: u32,
    pub sites: Range<u32>,
}
pub(super) enum OwnerRow {
    Body(Row),
    Declaration(DeclarationRow),
}
pub(crate) struct Expected {
    pub definition: DefId,
    pub kind: BodyKind,
    pub parent: Option<ImportedBodyId>,
    pub unit: ImportedBodyId,
}
pub(crate) struct Installed {
    epoch: Epoch,
    first: u32,
    len: u32,
}
impl Installed {
    pub(crate) fn body(&self, body: BodyRef) -> Result<ImportedBodyId, &'static str> {
        if body.0 >= self.len {
            return Err("body reference belongs outside this installed artifact");
        }
        Ok(ImportedBodyId(ImportedOwnerId {
            epoch: self.epoch.0,
            row: self.first + body.0,
        }))
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/catalog_freezing.rs"]
mod freezing_tests;
pub(crate) struct Catalog {
    pub(super) epoch: Epoch,
    pub(super) artifacts: Vec<Artifact>,
    pub(super) rows: Vec<OwnerRow>,
    pub(super) owners: HashMap<CrateIdentity, u32>,
    pub(super) definitions: HashMap<DefId, ImportedDeclarationId>,
    pub(super) bytes: usize,
    work: Cell<usize>,
    pub(super) limits: Limits,
    frozen: bool,
}
#[derive(Default)]
pub(crate) struct CatalogSlot(Option<Rc<Catalog>>);
impl CatalogSlot {
    pub(crate) fn get(&self) -> Option<&Catalog> {
        self.0.as_deref()
    }
    pub(crate) fn ensure(&mut self) -> Result<&mut Catalog, &'static str> {
        if self.0.is_none() {
            let mut catalog = Catalog::new(Limits::default())?;
            // The single owning catalog pays its Rc control block once. Clones
            // share this payload; ordinary source-only compilation has no Rc.
            catalog.bytes = catalog.bytes.saturating_add(2 * size_of::<usize>());
            if catalog.bytes > catalog.limits.bytes {
                return Err("imported catalog shared header exceeds storage budget");
            }
            self.0 = Some(Rc::new(catalog));
        }
        let catalog = self.0.as_mut().ok_or("imported body catalog disappeared")?;
        catalog.mutable()?;
        Rc::get_mut(catalog).ok_or("unfrozen imported catalog has foreign shared ownership")
    }
    pub(crate) fn freeze(&mut self) -> Result<(), &'static str> {
        let Some(catalog) = self.0.as_mut() else {
            return Ok(());
        };
        catalog.charge(1)?;
        if !catalog.frozen {
            Rc::get_mut(catalog)
                .ok_or("unfrozen imported catalog has foreign shared ownership")?
                .frozen = true;
        }
        Ok(())
    }
    pub(crate) fn shared(&self) -> Result<Option<Rc<Catalog>>, &'static str> {
        let Some(catalog) = self.0.as_ref() else {
            return Ok(None);
        };
        catalog.charge(1)?;
        if !catalog.frozen {
            return Err("imported catalog must be frozen before sharing");
        }
        Ok(Some(Rc::clone(catalog)))
    }
}
impl Catalog {
    pub(crate) fn new(limits: Limits) -> Result<Self, &'static str> {
        if size_of::<Self>() > limits.bytes {
            return Err("imported body catalog exceeds storage budget");
        }
        Ok(Self {
            epoch: Epoch::allocate(&NEXT_CATALOG)?,
            artifacts: Vec::new(),
            rows: Vec::new(),
            owners: HashMap::default(),
            definitions: HashMap::default(),
            bytes: size_of::<Self>(),
            work: Cell::new(0),
            limits,
            frozen: false,
        })
    }
    pub(super) fn charge(&self, steps: usize) -> Result<(), &'static str> {
        let total = self.work.get().saturating_add(steps);
        self.work.set(total);
        if total > self.limits.steps {
            return Err("imported body catalog work budget exhausted");
        }
        Ok(())
    }
    pub(crate) fn work(&self) -> usize {
        self.work.get()
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.bytes
    }
    pub(super) fn mutable(&self) -> Result<(), &'static str> {
        self.charge(1)?;
        if self.frozen {
            return Err("imported catalog is permanently frozen");
        }
        Ok(())
    }
    pub(crate) fn body_count(&self) -> usize {
        self.rows.len() - self.definitions.len()
    }
    pub(super) fn owner(&self, id: ImportedOwnerId) -> Result<&OwnerRow, &'static str> {
        self.charge(1)?;
        if id.epoch != self.epoch.0 {
            return Err("imported owner belongs to a foreign or retired catalog");
        }
        self.rows
            .get(id.row as usize)
            .ok_or("imported owner row is missing or stale")
    }
    pub(super) fn get(&self, id: ImportedBodyId) -> Result<&Row, &'static str> {
        match self.owner(id.0)? {
            OwnerRow::Body(row) => Ok(row),
            OwnerRow::Declaration(_) => Err("imported body handle names a declaration row"),
        }
    }
    pub(crate) fn validate(
        &self,
        id: ImportedBodyId,
        expected: &Expected,
    ) -> Result<BodyOrigin, &'static str> {
        let row = self.get(id)?;
        if row.definition != expected.definition
            || row.kind != expected.kind
            || row.parent != expected.parent
            || row.unit != expected.unit
        {
            return Err("imported body changed its declaration, kind, parent or numbering unit");
        }
        Ok(BodyOrigin::Imported(id))
    }
    pub(crate) fn coordinate(
        &self,
        id: ImportedBodyId,
    ) -> Result<(&CrateIdentity, BodyRef, Edition), &'static str> {
        let row = self.get(id)?;
        let artifact = &self.artifacts[row.artifact as usize];
        Ok((&artifact.owner, row.body, artifact.edition))
    }
    pub(crate) fn install(
        &mut self,
        definitions: &[Definition],
        input: Input,
    ) -> Result<Installed, &'static str> {
        super::publication::install(self, definitions, input)
    }
}
impl Installed {
    pub(super) fn new(epoch: Epoch, first: u32, len: u32) -> Self {
        Self { epoch, first, len }
    }
}
