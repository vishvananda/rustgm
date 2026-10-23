//! Additive envelopes retain each earlier payload's byte contract.
use super::{Abi, Interface, Reader, Writer};
use crate::support::crates::{RecordRepr, TypeGraph, TypeNode};

const PUBLIC_NAMES: u8 = 6;
const AGGREGATES: u8 = 7;
const VARIADICS: u8 = 8;
const GENERIC_ORIGINALS: u8 = 9;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Payload {
    Scalar,
    Values,
    Borrowed,
    Records,
    Foreign,
    Storage,
}
impl Payload {
    const fn tag(self) -> u8 {
        match self {
            Self::Scalar => 0,
            Self::Values => 1,
            Self::Borrowed => 2,
            Self::Records => 3,
            Self::Foreign => 4,
            Self::Storage => 5,
        }
    }
    fn decode(tag: u8) -> Result<Self, &'static str> {
        match tag {
            0 => Ok(Self::Scalar),
            1 => Ok(Self::Values),
            2 => Ok(Self::Borrowed),
            3 => Ok(Self::Records),
            4 => Ok(Self::Foreign),
            5 => Ok(Self::Storage),
            _ => Err("invalid metadata base profile"),
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct Profile {
    pub(super) payload: Payload,
    pub(super) public_names: bool,
    pub(super) shared: bool,
    pub(super) aggregates: bool,
    calling: Calling,
    originals: Originals,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Originals {
    None,
    Generic,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Calling {
    Fixed,
    VariadicC,
}
impl Calling {
    fn for_interface(interface: &Interface) -> Self {
        if interface
            .functions()
            .iter()
            .any(|f| f.signature().variadic())
        {
            Self::VariadicC
        } else {
            Self::Fixed
        }
    }
}
impl Profile {
    pub(super) fn for_interface(interface: &Interface) -> Self {
        let shared = interface.nominals().is_some();
        let foreign = interface
            .functions()
            .iter()
            .any(|f| f.signature().abi() != Abi::Rust);
        let payload = if interface.storage().is_some() {
            Payload::Storage
        } else if foreign {
            Payload::Foreign
        } else if shared {
            Payload::Records
        } else if interface
            .functions()
            .iter()
            .any(|f| !f.signature().is_scalar())
        {
            Payload::Borrowed
        } else if interface.constants().is_empty() {
            Payload::Scalar
        } else {
            Payload::Values
        };
        Self {
            payload,
            public_names: interface.public_names().is_some(),
            shared,
            aggregates: aggregate_types(interface),
            calling: Calling::for_interface(interface),
            originals: if interface.generic_payload().is_some() {
                Originals::Generic
            } else {
                Originals::None
            },
        }
    }
    pub(super) fn write(self, out: &mut Writer) -> Result<(), &'static str> {
        if self.generic() {
            out.u8(GENERIC_ORIGINALS)?;
        }
        if self.variadic() {
            out.u8(VARIADICS)?;
        }
        if self.aggregates {
            out.u8(AGGREGATES)?;
        }
        if self.public_names {
            out.u8(PUBLIC_NAMES)?;
        }
        out.u8(self.payload.tag())?;
        if self.payload == Payload::Foreign {
            out.u8(u8::from(self.shared))?;
        }
        Ok(())
    }
    pub(super) fn read(input: &mut Reader<'_>) -> Result<Self, &'static str> {
        let mut tag = input.u8()?;
        let generic = tag == GENERIC_ORIGINALS;
        if generic {
            tag = input.u8()?;
        }
        let variadics = tag == VARIADICS;
        if variadics {
            tag = input.u8()?;
        }
        let aggregates = tag == AGGREGATES;
        if aggregates {
            tag = input.u8()?;
        }
        let public_names = tag == PUBLIC_NAMES;
        if public_names {
            tag = input.u8()?;
        }
        let payload = Payload::decode(tag)?;
        let shared = if payload == Payload::Foreign {
            match input.u8()? {
                0 => false,
                1 => true,
                _ => return Err("unknown metadata graph ownership"),
            }
        } else {
            matches!(payload, Payload::Records | Payload::Storage)
        };
        Ok(Self {
            payload,
            public_names,
            shared,
            aggregates,
            originals: if generic {
                Originals::Generic
            } else {
                Originals::None
            },
            calling: if variadics {
                Calling::VariadicC
            } else {
                Calling::Fixed
            },
        })
    }
    pub(super) fn generic(self) -> bool {
        self.originals == Originals::Generic
    }
    pub(super) fn typed(self) -> bool {
        self.payload == Payload::Borrowed || (self.payload == Payload::Foreign && !self.shared)
    }
    pub(super) fn foreign(self) -> bool {
        matches!(self.payload, Payload::Foreign | Payload::Storage)
    }
    pub(super) fn variadic(self) -> bool {
        self.calling == Calling::VariadicC
    }
    pub(super) fn validate(self, interface: &Interface) -> Result<(), &'static str> {
        if self.generic() != interface.generic_payload().is_some() {
            return Err("generic metadata envelope requires complete originals");
        }
        if self.calling != Calling::for_interface(interface) {
            return Err("variadic metadata profile requires a variadic function");
        }
        if self.aggregates != aggregate_types(interface) {
            return Err("aggregate metadata profile requires array or representation facts");
        }
        Ok(())
    }
}

fn aggregate_types(interface: &Interface) -> bool {
    let arrays = |graph: &TypeGraph| {
        graph
            .nodes()
            .iter()
            .any(|node| matches!(node, TypeNode::Array { .. }))
    };
    if let Some(nominals) = interface.nominals() {
        arrays(nominals.types())
            || nominals
                .records()
                .iter()
                .any(|record| record.repr() != RecordRepr::Rust)
    } else {
        interface
            .functions()
            .iter()
            .any(|function| arrays(function.signature().types()))
    }
}
