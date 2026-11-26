//! The foreign boundary of the bridge: the declarations of the pinned CPPGM
//! `LowIR` library and the only `unsafe` blocks of this crate.
//!
//! A [`Module`] owns one library module from creation to destruction. Every
//! method takes Rust-owned slices and plain numbers; the library copies
//! names, parameters, initializers and templates, and body records are copied
//! into the pools it reserves. A failure poisons the library module: read the
//! message and drop the [`Module`].
#![expect(
    unsafe_code,
    reason = "the pinned CPPGM LowIR library is a C interface; this module alone calls it"
)]
use crate::records::{
    ABI_VERSION, ASM_DIALECT_ATT, ASM_DIALECT_INTEL, ArgumentFact, AsmAlternative, AsmGroup,
    AsmOperand, AsmPiece, AsmRegister, BINDING_DEFAULT, BodyCounts, Boundary, DataItem,
    Instruction, Literal, NO_ID, Parameter, Slot, TARGET_LINUX, Type, Value,
};
use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    mem::size_of,
    path::Path,
    ptr::{self, NonNull},
};

/// `CppgmLowir`: the library's module, opaque to its clients.
type Opaque = c_void;
/// `CppgmBodyBuffers`: the pools the library reserves for one body.
#[repr(C)]
struct BodyBuffers {
    instructions: *mut Instruction,
    operands: *mut Value,
    literals: *mut Literal,
    slots: *mut Slot,
    layout: *mut u32,
    capacity: BodyCounts,
}
/// `CppgmAssembly`: one template as flat arrays with their counts.
#[repr(C)]
struct Assembly {
    dialect: u8,
    memory: u8,
    properties: u16,
    state_read: u32,
    state_write: u32,
    argument_count: u32,
    location: *const c_void,
    text: *const c_char,
    operands: *const AsmOperand,
    alternatives: *const AsmAlternative,
    groups: *const AsmGroup,
    registers: *const AsmRegister,
    immediates: *const c_void,
    pieces: *const AsmPiece,
    clobbers: *const c_void,
    text_size: u32,
    operand_count: u32,
    alternative_count: u32,
    group_count: u32,
    register_count: u32,
    immediate_count: u32,
    piece_count: u32,
    clobber_count: u32,
}
const _: [(); 64] = [(); size_of::<BodyBuffers>()];
const _: [(); 120] = [(); size_of::<Assembly>()];

unsafe extern "C" {
    fn cppgm_lowir_abi_version() -> u32;
    fn cppgm_lowir_version() -> *const c_char;
    fn cppgm_lowir_create(abi_version: u32, target: u32) -> *mut Opaque;
    fn cppgm_lowir_destroy(module: *mut Opaque);
    fn cppgm_lowir_error(module: *const Opaque) -> *const c_char;
    fn cppgm_lowir_symbol(module: *mut Opaque, name: *const c_char, length: u32) -> u32;
    fn cppgm_lowir_boundary(
        module: *mut Opaque,
        result: Type,
        parameters: *const Parameter,
        count: u32,
        boundary: *const Boundary,
    ) -> u32;
    fn cppgm_lowir_function(
        module: *mut Opaque,
        symbol: u32,
        signature: u32,
        binding: u32,
        definition: u32,
    ) -> c_int;
    fn cppgm_lowir_global(
        module: *mut Opaque,
        symbol: u32,
        ty: Type,
        binding: u32,
        storage: u32,
        items: *const DataItem,
        count: u32,
        declaration: u32,
    ) -> c_int;
    fn cppgm_lowir_symbol_fact(module: *mut Opaque, symbol: u32, key: u32, value: u32) -> c_int;
    fn cppgm_lowir_body_reserve(
        module: *mut Opaque,
        symbol: u32,
        minimum: *const BodyCounts,
        buffers: *mut BodyBuffers,
    ) -> c_int;
    fn cppgm_lowir_body_commit(
        module: *mut Opaque,
        symbol: u32,
        counts: *const BodyCounts,
        temp_count: u32,
    ) -> c_int;
    fn cppgm_lowir_body_argument_facts(
        module: *mut Opaque,
        symbol: u32,
        facts: *const ArgumentFact,
        count: u32,
    ) -> c_int;
    fn cppgm_lowir_assembly(module: *mut Opaque, assembly: *const Assembly) -> u32;
    fn cppgm_lowir_validate(module: *mut Opaque) -> c_int;
    fn cppgm_lowir_write(module: *mut Opaque, path: *const c_char) -> c_int;
    fn cppgm_lowir_compile(module: *mut Opaque, level: u32, pic: u32) -> c_int;
    fn cppgm_lowir_object_data(module: *const Opaque) -> *const u8;
    fn cppgm_lowir_object_size(module: *const Opaque) -> u64;
}

/// What the library reported when it refused a call.
#[derive(Debug)]
pub struct Rejected(pub String);

/// The records of one function body, in the library's pool order.
pub struct Body<'a> {
    pub instructions: &'a [Instruction],
    pub operands: &'a [Value],
    pub literals: &'a [Literal],
    pub slots: &'a [Slot],
    /// The block identities in layout order.
    pub layout: &'a [u32],
    /// Parameters first, then every defined temporary.
    pub temporaries: u32,
}
/// One assembly template with its dialect, memory effect and invocation arity.
pub struct Template<'a> {
    pub att: bool,
    pub memory: u8,
    pub arguments: u32,
    pub properties: u16,
    pub reads: u32,
    pub writes: u32,
    pub text: &'a [u8],
    pub operands: &'a [AsmOperand],
    pub alternatives: &'a [AsmAlternative],
    pub groups: &'a [AsmGroup],
    pub registers: &'a [AsmRegister],
    pub pieces: &'a [AsmPiece],
}

fn count(length: usize) -> Result<u32, Rejected> {
    u32::try_from(length).map_err(|_| Rejected("record count exceeds the LowIR library's".into()))
}

/// Copy `source` into a pool the library reserved for `capacity` records.
fn fill<T: Copy>(pool: *mut T, capacity: u32, source: &[T]) -> Result<(), Rejected> {
    if source.is_empty() {
        return Ok(());
    }
    if pool.is_null() || (capacity as usize) < source.len() {
        return Err(Rejected(
            "the LowIR library reserved a smaller body than asked".into(),
        ));
    }
    // SAFETY: `pool` came from a successful `cppgm_lowir_body_reserve` and no
    // library call followed it, so it is valid for `capacity` writes of `T`;
    // the check above bounds `source.len()` by `capacity`. The pool is owned
    // by the library and cannot overlap a Rust slice.
    unsafe { ptr::copy_nonoverlapping(source.as_ptr(), pool, source.len()) };
    Ok(())
}

/// One library module and, after compilation, its object.
pub struct Module {
    raw: NonNull<Opaque>,
    compiled: bool,
}
impl Module {
    /// Create a module for x86-64 Linux.
    ///
    /// # Errors
    /// Refuses a library of another interface version or release.
    pub fn create(release: &str) -> Result<Self, Rejected> {
        // SAFETY: the function takes no argument and needs no module.
        let version = unsafe { cppgm_lowir_abi_version() };
        if version != ABI_VERSION {
            return Err(Rejected(format!(
                "the LowIR library states interface version {version}; this bridge is written for {ABI_VERSION}"
            )));
        }
        // SAFETY: the function takes no argument and needs no module.
        let reported = unsafe { cppgm_lowir_version() };
        if reported.is_null() {
            return Err(Rejected("the LowIR library states no release".into()));
        }
        // SAFETY: the library returns a NUL-terminated string with static
        // storage; it was checked non-null and is copied before any other call.
        let reported = unsafe { CStr::from_ptr(reported) }.to_string_lossy();
        if reported != release {
            return Err(Rejected(format!(
                "the LowIR library is release {reported}; the backend lock names {release}"
            )));
        }
        // SAFETY: creation takes two numbers and returns null on failure.
        let raw = unsafe { cppgm_lowir_create(ABI_VERSION, TARGET_LINUX) };
        NonNull::new(raw)
            .map(|raw| Self {
                raw,
                compiled: false,
            })
            .ok_or_else(|| Rejected("the LowIR library created no module".into()))
    }
    fn failure(&self) -> Rejected {
        // SAFETY: `raw` is the live module this value owns.
        let message = unsafe { cppgm_lowir_error(self.raw.as_ptr()) };
        if message.is_null() {
            return Rejected("the LowIR library failed without a diagnostic".into());
        }
        // SAFETY: the diagnostic is non-null, NUL-terminated and owned by the
        // live module; it is copied here, before any further library call.
        let message = unsafe { CStr::from_ptr(message) }.to_string_lossy();
        Rejected(message.into_owned())
    }
    fn status(&self, status: c_int) -> Result<(), Rejected> {
        if status == 0 {
            Err(self.failure())
        } else {
            Ok(())
        }
    }
    fn identity(&self, id: u32) -> Result<u32, Rejected> {
        if id == NO_ID {
            Err(self.failure())
        } else {
            Ok(id)
        }
    }
    /// Intern a top-level name; the same name gives the same identity.
    ///
    /// # Errors
    /// Reports the library's refusal.
    pub fn symbol(&mut self, name: &str) -> Result<u32, Rejected> {
        let length = count(name.len())?;
        // SAFETY: the pointer and length describe the live `name`; the library
        // copies the bytes and does not need a terminator.
        let id = unsafe { cppgm_lowir_symbol(self.raw.as_ptr(), name.as_ptr().cast(), length) };
        self.identity(id)
    }
    /// Add one signature with its boundary metadata.
    ///
    /// # Errors
    /// Reports the library's refusal.
    pub fn boundary(
        &mut self,
        result: Type,
        parameters: &[Parameter],
        boundary: &Boundary,
    ) -> Result<u32, Rejected> {
        let length = count(parameters.len())?;
        // SAFETY: `parameters` is live for `length` records and `boundary` is
        // a live record; the library copies both.
        let id = unsafe {
            cppgm_lowir_boundary(
                self.raw.as_ptr(),
                result,
                parameters.as_ptr(),
                length,
                ptr::from_ref(boundary),
            )
        };
        self.identity(id)
    }
    /// Declare or define a function with the default binding.
    ///
    /// # Errors
    /// Reports the library's refusal.
    pub fn function(
        &mut self,
        symbol: u32,
        signature: u32,
        definition: bool,
    ) -> Result<(), Rejected> {
        // SAFETY: the call takes the live module and plain numbers.
        let status = unsafe {
            cppgm_lowir_function(
                self.raw.as_ptr(),
                symbol,
                signature,
                BINDING_DEFAULT,
                u32::from(definition),
            )
        };
        self.status(status)
    }
    /// Declare or define a global with the default binding and a
    /// `STORAGE_*` storage.
    ///
    /// # Errors
    /// Reports the library's refusal.
    pub fn global(
        &mut self,
        symbol: u32,
        ty: Type,
        storage: u32,
        items: &[DataItem],
        declaration: bool,
    ) -> Result<(), Rejected> {
        let length = count(items.len())?;
        // SAFETY: `items` is live for `length` records; the library copies them.
        let status = unsafe {
            cppgm_lowir_global(
                self.raw.as_ptr(),
                symbol,
                ty,
                BINDING_DEFAULT,
                storage,
                items.as_ptr(),
                length,
                u32::from(declaration),
            )
        };
        self.status(status)
    }
    /// State one numeric fact of a declared symbol.
    ///
    /// # Errors
    /// Reports the library's refusal.
    pub fn symbol_fact(&mut self, symbol: u32, key: u32, value: u32) -> Result<(), Rejected> {
        // SAFETY: the call takes the live module and plain numbers.
        let status = unsafe { cppgm_lowir_symbol_fact(self.raw.as_ptr(), symbol, key, value) };
        self.status(status)
    }
    /// Reserve the pools of one defined function, fill them and commit.
    ///
    /// # Errors
    /// Reports the library's refusal or a reservation smaller than asked.
    pub fn body(&mut self, symbol: u32, body: &Body<'_>) -> Result<(), Rejected> {
        let counts = BodyCounts {
            instructions: count(body.instructions.len())?,
            operands: count(body.operands.len())?,
            literals: count(body.literals.len())?,
            slots: count(body.slots.len())?,
            blocks: count(body.layout.len())?,
        };
        let mut buffers = BodyBuffers {
            instructions: ptr::null_mut(),
            operands: ptr::null_mut(),
            literals: ptr::null_mut(),
            slots: ptr::null_mut(),
            layout: ptr::null_mut(),
            capacity: BodyCounts::default(),
        };
        // SAFETY: `counts` and `buffers` are live locals of the stated record
        // types; the library reads the first and fills the second.
        let status = unsafe {
            cppgm_lowir_body_reserve(
                self.raw.as_ptr(),
                symbol,
                ptr::from_ref(&counts),
                ptr::from_mut(&mut buffers),
            )
        };
        self.status(status)?;
        let capacity = buffers.capacity;
        fill(
            buffers.instructions,
            capacity.instructions,
            body.instructions,
        )?;
        fill(buffers.operands, capacity.operands, body.operands)?;
        fill(buffers.literals, capacity.literals, body.literals)?;
        fill(buffers.slots, capacity.slots, body.slots)?;
        fill(buffers.layout, capacity.blocks, body.layout)?;
        // SAFETY: `counts` is a live record naming exactly the records written
        // above into the pools this module reserved for `symbol`.
        let status = unsafe {
            cppgm_lowir_body_commit(
                self.raw.as_ptr(),
                symbol,
                ptr::from_ref(&counts),
                body.temporaries,
            )
        };
        self.status(status)
    }
    /// State the classes of call arguments of a committed body.
    ///
    /// # Errors
    /// Reports the library's refusal.
    pub fn argument_facts(&mut self, symbol: u32, facts: &[ArgumentFact]) -> Result<(), Rejected> {
        let length = count(facts.len())?;
        // SAFETY: `facts` is live for `length` records; the library copies them.
        let status = unsafe {
            cppgm_lowir_body_argument_facts(self.raw.as_ptr(), symbol, facts.as_ptr(), length)
        };
        self.status(status)
    }
    /// Define one assembly template and return its identity.
    ///
    /// # Errors
    /// Reports the library's refusal.
    pub fn assembly(&mut self, template: &Template<'_>) -> Result<u32, Rejected> {
        let record = Assembly {
            dialect: if template.att {
                ASM_DIALECT_ATT
            } else {
                ASM_DIALECT_INTEL
            },
            memory: template.memory,
            properties: template.properties,
            state_read: template.reads,
            state_write: template.writes,
            argument_count: template.arguments,
            location: ptr::null(),
            text: template.text.as_ptr().cast(),
            operands: template.operands.as_ptr(),
            alternatives: template.alternatives.as_ptr(),
            groups: template.groups.as_ptr(),
            registers: template.registers.as_ptr(),
            immediates: ptr::null(),
            pieces: template.pieces.as_ptr(),
            clobbers: ptr::null(),
            text_size: count(template.text.len())?,
            operand_count: count(template.operands.len())?,
            alternative_count: count(template.alternatives.len())?,
            group_count: count(template.groups.len())?,
            register_count: count(template.registers.len())?,
            immediate_count: 0,
            piece_count: count(template.pieces.len())?,
            clobber_count: 0,
        };
        // SAFETY: `record` is a live record whose every array pointer names a
        // live slice of `template` with exactly the count stated beside it;
        // the absent arrays are null with a zero count. The library copies all.
        let id = unsafe { cppgm_lowir_assembly(self.raw.as_ptr(), ptr::from_ref(&record)) };
        self.identity(id)
    }
    /// Validate the constructed module.
    ///
    /// # Errors
    /// Reports the library's diagnostic.
    pub fn validate(&mut self) -> Result<(), Rejected> {
        // SAFETY: the call takes only the live module.
        let status = unsafe { cppgm_lowir_validate(self.raw.as_ptr()) };
        self.status(status)
    }
    /// Write the library's own text view of the module to `path`.
    ///
    /// # Errors
    /// Reports an unrepresentable path or the library's refusal.
    pub fn write(&mut self, path: &Path) -> Result<(), Rejected> {
        let path = CString::new(path.as_os_str().as_encoded_bytes())
            .map_err(|_| Rejected("the LowIR view path holds a NUL byte".into()))?;
        // SAFETY: `path` is a live NUL-terminated string for the whole call.
        let status = unsafe { cppgm_lowir_write(self.raw.as_ptr(), path.as_ptr()) };
        self.status(status)
    }
    /// Compile the module once into a relocatable object.
    ///
    /// # Errors
    /// Reports the library's diagnostic.
    pub fn compile(&mut self, level: u32, pic: bool) -> Result<(), Rejected> {
        // SAFETY: the call takes the live module and plain numbers.
        let status = unsafe { cppgm_lowir_compile(self.raw.as_ptr(), level, u32::from(pic)) };
        self.status(status)?;
        self.compiled = true;
        Ok(())
    }
    /// The bytes of the compiled object, owned by the module.
    ///
    /// # Errors
    /// Reports a module that was not compiled or that holds no object.
    pub fn object(&self) -> Result<&[u8], Rejected> {
        if !self.compiled {
            return Err(Rejected("the LowIR module was not compiled".into()));
        }
        // SAFETY: the call only reads the live module.
        let data = unsafe { cppgm_lowir_object_data(self.raw.as_ptr()) };
        // SAFETY: the call only reads the live module.
        let size = unsafe { cppgm_lowir_object_size(self.raw.as_ptr()) };
        let size = usize::try_from(size)
            .map_err(|_| Rejected("the LowIR object exceeds the address space".into()))?;
        if data.is_null() || size == 0 {
            return Err(Rejected("the LowIR library made an empty object".into()));
        }
        // SAFETY: after a successful compilation the library keeps `size`
        // initialized bytes at `data` until the module is destroyed. Only
        // `drop` destroys it and compilation is allowed once, so the buffer
        // outlives the shared borrow of `self` this slice carries.
        Ok(unsafe { std::slice::from_raw_parts(data, size) })
    }
}
impl Drop for Module {
    fn drop(&mut self) {
        // SAFETY: `raw` is the module this value created and owns; it is
        // destroyed exactly once, here.
        unsafe { cppgm_lowir_destroy(self.raw.as_ptr()) };
    }
}
