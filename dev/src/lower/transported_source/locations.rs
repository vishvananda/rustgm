//! Dense body-local labels preserve real source provenance without serializing it.
use super::{Body, HashMap, Stage, ir, retained, s};
use ir::{BodyOrigin, BodySite, ProofSite};

pub(super) struct Sites {
    source: BodyOrigin,
    rows: HashMap<BodySite, u32>,
    formations: Option<Box<Formations>>,
}
#[derive(Default)]
struct Formations {
    by_type: HashMap<crate::types::TypeId, usize>,
    rows: Vec<Option<s::Obligation>>,
}
impl Sites {
    pub(super) fn new(stage: &mut Stage, body: &Body) -> Result<Self, &'static str> {
        let mut sites = Self {
            source: body.source,
            rows: HashMap::default(),
            formations: None,
        };
        sites.body(stage, body.source.root_body_site())?;
        Ok(sites)
    }
    pub(super) fn body(&mut self, stage: &mut Stage, site: BodySite) -> Result<u32, &'static str> {
        stage.charge(1)?;
        self.source.proof_site(site)?;
        if let Some(&index) = self.rows.get(&site) {
            return Ok(index);
        }
        stage.reserve(retained::new_map::<BodySite, u32>(1))?;
        let index = self.len()?;
        self.rows.insert(site, index);
        Ok(index)
    }
    pub(super) fn input(
        &mut self,
        stage: &mut Stage,
        site: ProofSite,
    ) -> Result<s::InputSite, &'static str> {
        stage.charge(1)?;
        let source = site
            .source()
            .ok_or("source original input has no actual source location")?;
        if self
            .source
            .source()
            .is_none_or(|owner| source.file != owner.file)
        {
            return Err(
                "cross-unit declaration input sites require explicit declaration location transport",
            );
        }
        let index = self.body(stage, source.node.into())?;
        Ok(s::InputSite::Body(index))
    }
    pub(super) fn len(&self) -> Result<u32, &'static str> {
        u32::try_from(self.rows.len()).map_err(|_| "source original location table overflow")
    }
}

impl Sites {
    pub(super) fn function_formation(
        &mut self,
        stage: &mut Stage,
        root: BodySite,
        ty: crate::types::TypeId,
        wire: crate::support::crates::generics::TypeRef,
    ) -> Result<(), &'static str> {
        stage.charge(1)?;
        if self
            .formations
            .as_deref()
            .is_some_and(|rows| rows.by_type.contains_key(&ty))
        {
            return Ok(());
        }
        if self.formations.is_none() {
            stage.reserve(size_of::<Formations>())?;
            self.formations = Some(Box::default());
        }
        let rows = self
            .formations
            .as_deref()
            .ok_or("source formation collector disappeared")?;
        let map = retained::Growth::map::<crate::types::TypeId, usize>(
            rows.by_type.len(),
            rows.by_type.capacity(),
            1,
        );
        let vector = retained::Growth::vector::<Option<s::Obligation>>(
            rows.rows.len(),
            rows.rows.capacity(),
            1,
        );
        stage.charge(map.work.saturating_add(vector.work))?;
        stage.reserve(map.bytes.saturating_add(vector.bytes))?;
        let site = self.body(stage, root)?;
        let rows = self
            .formations
            .as_deref_mut()
            .ok_or("source formation collector disappeared")?;
        rows.by_type.insert(ty, rows.rows.len());
        rows.rows.push(Some(s::Obligation {
            site: s::InputSite::Body(site),
            kind: s::InputKind::WellFormed(wire),
        }));
        Ok(())
    }
    pub(super) fn existing_formation(
        &mut self,
        stage: &mut Stage,
        ty: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        stage.charge(1)?;
        if let Some(rows) = self.formations.as_deref_mut()
            && let Some(index) = rows.by_type.remove(&ty)
        {
            rows.rows[index] = None;
        }
        Ok(())
    }
    pub(super) fn append_formations(
        &mut self,
        stage: &mut Stage,
        obligations: &mut Vec<s::Obligation>,
    ) -> Result<(), &'static str> {
        let Some(rows) = self.formations.take() else {
            return Ok(());
        };
        stage.charge(rows.rows.len())?;
        let growth = retained::Growth::vector::<s::Obligation>(
            obligations.len(),
            obligations.capacity(),
            rows.by_type.len(),
        );
        stage.charge(growth.work)?;
        stage.reserve(growth.bytes)?;
        obligations.extend(rows.rows.into_iter().flatten());
        Ok(())
    }
}
