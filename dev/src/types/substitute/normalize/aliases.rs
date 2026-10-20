//! Distinct associated operations retain their complete inputs through the shared fold.
use crate::types::{GenericArgs, Kind, opaque::OpaqueId, projection::Application};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Alias {
    Associated(Application),
    TraitResult(OpaqueId, GenericArgs),
}
impl Alias {
    #[must_use]
    pub const fn kind(self) -> Kind {
        match self {
            Self::Associated(application) => Kind::Projection(application),
            Self::TraitResult(id, arguments) => Kind::Opaque(id, arguments),
        }
    }
    #[must_use]
    pub const fn arguments(self) -> GenericArgs {
        match self {
            Self::Associated(application) => application.arguments,
            Self::TraitResult(_, arguments) => arguments,
        }
    }
    #[must_use]
    pub const fn with_arguments(self, arguments: GenericArgs) -> Self {
        match self {
            Self::Associated(application) => Self::Associated(Application {
                arguments,
                ..application
            }),
            Self::TraitResult(id, _) => Self::TraitResult(id, arguments),
        }
    }
    #[must_use]
    pub const fn associated(self) -> Option<Application> {
        match self {
            Self::Associated(application) => Some(application),
            Self::TraitResult(..) => None,
        }
    }
}
