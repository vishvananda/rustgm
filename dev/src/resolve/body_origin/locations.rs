//! Provider labels cannot create a source node or change operation identity.
use super::{BodySite, Catalog, ImportedProofSite, ProofSite, catalog::OwnerRow};

pub(crate) struct ProviderFile {
    pub path: String,
    pub bytes: u32,
}
#[derive(Clone, Copy)]
pub(crate) struct Label {
    pub file: u32,
    pub start: u32,
    pub end: u32,
}
pub(crate) struct Location<'a> {
    pub file: &'a str,
    pub start: u32,
    pub end: u32,
}
impl Catalog {
    pub(crate) fn site(
        &self,
        body: super::ImportedBodyId,
        index: u32,
    ) -> Result<BodySite, &'static str> {
        let row = self.get(body)?;
        self.charge(1)?;
        if index >= row.sites.end - row.sites.start {
            return Err("operation site belongs outside its imported body");
        }
        BodySite::checked(index)
    }
    pub(crate) fn proof_site(
        &self,
        body: super::ImportedBodyId,
        site: BodySite,
    ) -> Result<ProofSite, &'static str> {
        let actual = self.site(body, site.index())?;
        Ok(ProofSite::Imported(ImportedProofSite {
            owner: body.0,
            ordinal: actual.0,
        }))
    }
    pub(crate) fn location(&self, site: ProofSite) -> Result<Option<Location<'_>>, &'static str> {
        let ProofSite::Imported(site) = site else {
            return Ok(None);
        };
        let (artifact, sites, declaration) = match self.owner(site.owner)? {
            OwnerRow::Body(row) => (row.artifact, &row.sites, false),
            OwnerRow::Declaration(row) => (row.artifact, &row.sites, true),
        };
        self.charge(1)?;
        let index = site.ordinal.get() - 1;
        if index >= sites.end - sites.start {
            return Err("diagnostic site belongs outside its imported owner");
        }
        let artifact = &self.artifacts[artifact as usize];
        let labels = if declaration {
            &artifact.declaration_labels
        } else {
            &artifact.labels
        };
        let label = labels[(sites.start + index) as usize];
        Ok(label.map(|label| Location {
            file: &artifact.files[label.file as usize].path,
            start: label.start,
            end: label.end,
        }))
    }
}

pub(super) fn validate(catalog: &Catalog, input: &super::Input) -> Result<(), &'static str> {
    for file in &input.files {
        catalog.charge(1 + file.path.len())?;
        if file.path.is_empty()
            || file.path.len() > 4096
            || file
                .path
                .bytes()
                .any(|byte| matches!(byte, 0 | b'\n' | b'\r'))
        {
            return Err("invalid imported provider diagnostic path");
        }
    }
    for label in input.labels.iter().chain(&input.declaration_labels) {
        catalog.charge(1)?;
        if let Some(label) = label {
            let file = input
                .files
                .get(label.file as usize)
                .ok_or("imported diagnostic label names an unknown provider file")?;
            if label.end < label.start || label.end > file.bytes {
                return Err("imported diagnostic label is outside its provider file");
            }
        }
    }
    Ok(())
}
