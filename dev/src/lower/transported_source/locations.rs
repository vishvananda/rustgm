//! Dense body-local labels preserve real source provenance without serializing it.
use super::{Body, HashMap, Stage, ir, retained, s};
use ir::{BodyOrigin, BodySite, ProofSite};

pub(super) struct Sites {
    source: BodyOrigin,
    rows: HashMap<BodySite, u32>,
}
impl Sites {
    pub(super) fn new(stage: &mut Stage, body: &Body) -> Result<Self, &'static str> {
        let mut sites = Self {
            source: body.source,
            rows: HashMap::default(),
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
