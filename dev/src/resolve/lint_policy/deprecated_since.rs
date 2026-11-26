//! Deprecation release data does not grant API availability or imported absence.
//! The staged rules are those of the crate that declares the deprecation.
use super::{Violation, expectations::Deprecation};
use crate::resolve::{package::Package, stability, unit::NodeKey};

#[derive(Clone, Copy)]
pub(crate) enum Since {
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
    fn active(self, staged: bool) -> bool {
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
        if !package.staged_crate(key.file) {
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
    pub(super) fn active(&self, package: &Package, key: NodeKey) -> bool {
        self.release.active(package.staged_crate(key.file))
    }
}

pub(crate) fn transported_active(
    since: &crate::support::crates::generics::source::DeprecationSince,
    staged: bool,
) -> bool {
    use crate::support::crates::generics::source::{AvailabilitySince, DeprecationSince};
    let release = match since {
        DeprecationSince::Future => Since::Future,
        DeprecationSince::Version(AvailabilitySince::Version {
            major,
            minor,
            patch,
        }) => Since::Version(stability::Since::Version {
            major: *major,
            minor: *minor,
            patch: *patch,
        }),
        _ => Since::Unspecified,
    };
    release.active(staged)
}
