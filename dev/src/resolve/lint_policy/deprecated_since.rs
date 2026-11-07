//! Deprecation release data does not grant API availability or imported absence.
use super::{Violation, expectations::Deprecation};
use crate::resolve::{package::Package, stability, unit::NodeKey};

pub(super) enum Since {
    Unspecified,
    Future,
    Version(stability::Since),
    NonStandard,
}
impl Since {
    pub(super) fn parse(text: Option<&str>) -> Self {
        match text {
            None => Self::Unspecified,
            Some("TBD") => Self::Future,
            Some(text) => stability::parse_version(text).map_or(Self::NonStandard, Self::Version),
        }
    }
    fn active(&self, staged: bool) -> bool {
        match self {
            Self::Future => false,
            Self::Version(version) if staged => {
                version.release() <= stability::Since::Current.release()
            }
            _ => true,
        }
    }
}
impl Deprecation {
    pub(super) fn validate_staged(&self, package: &Package, key: NodeKey) -> Result<(), Violation> {
        if !package
            .library_policy()
            .is_some_and(stability::Library::is_staged)
        {
            return Ok(());
        }
        let message = match self.release {
            Since::Unspecified => Some("staged deprecation requires its since field"),
            Since::NonStandard => Some("staged deprecation requires a valid version or exact TBD"),
            _ if self.note.is_none() => Some("staged deprecation requires its note field"),
            _ => None,
        };
        message.map_or(Ok(()), |message| {
            Err(Violation {
                key,
                offset: self.span.start,
                message,
            })
        })
    }
    pub(super) fn active(&self, package: &Package) -> bool {
        self.release.active(
            package
                .library_policy()
                .is_some_and(stability::Library::is_staged),
        )
    }
}
