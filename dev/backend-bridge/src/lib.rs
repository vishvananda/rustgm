//! The in-process backend bridge: `RustGM`'s typed `LowIR` stated to the pinned
//! CPPGM `LowIR` library, which returns a relocatable object.
//!
//! The bridge states the module that the pinned text reader builds from the
//! text `RustGM`'s writer serializes for the same program: identities are
//! created in the order that text first mentions them, and each literal is
//! one pool entry with the bits the reader gives its spelling in the context
//! its instruction reads it. The writer's typed checks run first, so a
//! program the writer rejects is rejected here with the same message.
//!
//! A form the bridge does not state, including every form the text reader
//! itself refuses, is reported as [`Failure::Unsupported`] with the count of
//! each such form in the module; the caller then takes the text path.
mod ffi;
mod instructions;
mod literals;
mod records;
mod stater;

use rustgm::lowir::program::Program;
use std::{collections::BTreeMap, fmt, path::Path};

/// The cppgm release named by the backend lock this crate was built against.
pub const RELEASE: &str = env!("RUSTGM_BRIDGE_RELEASE");
/// The interface version of the records this crate states.
pub const ABI_VERSION: u32 = records::ABI_VERSION;

/// The forms of one module that the bridge does not state, by occurrence.
#[derive(Debug, Default)]
pub struct Forms {
    counts: BTreeMap<&'static str, u64>,
    total: u64,
}
impl Forms {
    fn note(&mut self, form: &'static str) {
        *self.counts.entry(form).or_default() += 1;
        self.total += 1;
    }
    /// The number of occurrences noted so far: a statement that noted one
    /// is not made, so that the library never sees a placeholder record.
    fn total(&self) -> u64 {
        self.total
    }
    /// Whether the module states no such form.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }
    /// Each form with its number of occurrences, by name.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, u64)> + '_ {
        self.counts.iter().map(|(&form, &count)| (form, count))
    }
}
impl fmt::Display for Forms {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, (form, count)) in self.iter().enumerate() {
            if index != 0 {
                write!(out, "; ")?;
            }
            write!(out, "{form} x{count}")?;
        }
        Ok(())
    }
}

/// Why a program did not become an object in process.
#[derive(Debug)]
pub enum Failure {
    /// The typed program is inconsistent; the text writer rejects it too.
    Program(String),
    /// Forms this bridge does not state; the text path decides them.
    Unsupported(Forms),
    /// The library refused a call or the module.
    Library(String),
}
impl fmt::Display for Failure {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Program(message) => write!(out, "{message}"),
            Self::Unsupported(forms) => write!(out, "forms the bridge does not state: {forms}"),
            Self::Library(message) => {
                write!(out, "the LowIR library refused the module: {message}")
            }
        }
    }
}
impl From<&'static str> for Failure {
    fn from(message: &'static str) -> Self {
        Self::Program(message.into())
    }
}
impl From<ffi::Rejected> for Failure {
    fn from(rejected: ffi::Rejected) -> Self {
        Self::Library(rejected.0)
    }
}

/// How to compile one module.
#[derive(Clone, Copy, Default)]
pub struct Request<'a> {
    /// The optimization level, 0 to 3.
    pub level: u32,
    /// Position-independent code instead of fixed.
    pub pic: bool,
    /// Where to write the library's own text view before it compiles.
    pub view: Option<&'a Path>,
}

/// A compiled module that owns its object bytes.
pub struct Object {
    module: ffi::Module,
}
impl Object {
    /// The relocatable ELF object.
    ///
    /// # Errors
    /// Reports a library that kept no object.
    pub fn bytes(&self) -> Result<&[u8], Failure> {
        Ok(self.module.object()?)
    }
}

/// State `program` to the library and compile it.
///
/// # Errors
/// Reports an inconsistent program, the forms the bridge does not state or
/// the library's refusal.
pub fn compile(program: &Program, request: &Request<'_>) -> Result<Object, Failure> {
    let mut stater = stater::Stater::new(program, ffi::Module::create(RELEASE)?);
    stater.globals()?;
    stater.templates()?;
    stater.functions()?;
    let mut module = stater.finish()?;
    module.validate()?;
    if let Some(view) = request.view {
        module.write(view)?;
    }
    module.compile(request.level, request.pic)?;
    Ok(Object { module })
}
