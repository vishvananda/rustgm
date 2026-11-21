//! Bring-up guest crates: further crate roots resolved inside one session.
//!
//! The session's own crate is the host, slot 0. Each guest is a later slot
//! whose root module has no parent, so `super` and restricted visibility stop
//! there. `crate`, `$crate`, exported macros, the extern prelude and the
//! standard prelude follow the crate that owns the module they are asked
//! from. Lang items, features and coherence stay those of the session. Each
//! crate's source edition stays with its units; an omitted guest edition inherits
//! the host's. A guest given cfg options of its own is configured with those
//! alone; every other crate sees the session's `--cfg` options. The bring-up
//! profile names the guests; a strict compilation has
//! none and takes none of these paths. Their state lives in the session's boxed source core,
//! so no package or resolution record grows for them.
use super::{
    bindings::Source,
    index::Resolver,
    lookup::Observations,
    model::{Access, Binding, DefId, DefKind, Definition, Namespace, Origin, Resolution, ScopeId},
    names::{NameId, Names},
    package::{Error, Module, ModuleId, Package},
    unit::{FileId, NodeKey, Unit},
};
use crate::{
    expand::marks::ItemContext,
    support::{crates::Guest, edition::Edition},
};

/// The crate that owns a module: the host, or a guest in driver order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CrateSlot(pub u32);
impl CrateSlot {
    pub const HOST: Self = Self(0);
    const fn index(self) -> usize {
        self.0 as usize
    }
    /// This crate's row among the guest roots; the host has none.
    const fn guest(self) -> Option<usize> {
        match self.0 {
            0 => None,
            slot => Some(slot as usize - 1),
        }
    }
}

/// The standard library whose prelude a crate's root attributes select.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Library {
    /// `#![no_core]`: no prelude and no injected crate.
    None,
    /// `#![no_std]`.
    Core,
    Std,
}
impl Library {
    const fn crate_name(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Core => Some("core"),
            Self::Std => Some("std"),
        }
    }
}

/// One crate of a session that has guests; the host is the first.
pub struct Member {
    pub name: NameId,
    pub root: ModuleId,
    /// The unspellable path root that this crate's `$crate` transcribes to.
    macro_root: NameId,
    pub library: Library,
    /// `#![no_main]`: the crate supplies no Rust entry point.
    pub no_main: bool,
    /// `#![feature(staged_api)]`: the crate's deprecations follow the
    /// staged rules.
    staged: bool,
}

/// The crates of a session that has guests, the crate of each source file,
/// and the roots that resolution has entered so far, in slot order.
pub struct Guests {
    members: Vec<Member>,
    files: Vec<CrateSlot>,
    roots: Vec<Root>,
}
impl Guests {
    /// Begin a guest session over the loaded host root.
    pub(super) fn open(package: &mut Package) -> Result<Self, Error> {
        if package.source_core().is_none() {
            return Err(Error::unlocated("guest crates require a source core"));
        }
        let host = package.crate_name.clone();
        let mut guests = Self {
            members: Vec::new(),
            files: vec![CrateSlot::HOST],
            roots: Vec::new(),
        };
        let name = guests.name(&mut package.names, &host)?;
        let macro_root = guests.macro_root(&mut package.names)?;
        guests.members.push(Member {
            name,
            root: ModuleId(0),
            macro_root,
            library: Library::None,
            no_main: false,
            staged: package
                .library_policy()
                .is_some_and(crate::resolve::stability::Library::is_staged),
        });
        Ok(guests)
    }
    fn name(&self, names: &mut Names, spelling: &str) -> Result<NameId, Error> {
        let name = names.intern(spelling).map_err(Error::unlocated)?;
        if self.members.iter().any(|member| member.name == name) {
            return Err(Error::unlocated(format!(
                "guest crate `{spelling}` repeats the name of an earlier crate of this session"
            )));
        }
        Ok(name)
    }
    /// The next member's `$crate` root: crate ids start at one, because id
    /// zero spells the `crate` keyword of whichever crate uses the macro.
    fn macro_root(&self, names: &mut Names) -> Result<NameId, Error> {
        let id = u32::try_from(self.members.len() + 1)
            .map_err(|_| Error::unlocated("too many guest crates"))?;
        names
            .intern(&super::crates::macro_root_spelling(id))
            .map_err(Error::unlocated)
    }
    /// Admit a parsed guest root as the next crate; the caller installs the
    /// returned root module at `module`, backed by `file`.
    pub(super) fn admit(
        &mut self,
        names: &mut Names,
        input: &Guest,
        unit: &Unit,
        (file, module): (FileId, ModuleId),
    ) -> Result<Module, Error> {
        let root = unit.configured.ast.root;
        let word = |attribute: &str| {
            unit.word_attribute(Some(root), attribute)
                .map_err(|error| Error::at(file, error.offset, error.message))
        };
        let slot = CrateSlot(
            u32::try_from(self.members.len())
                .map_err(|_| Error::unlocated("too many guest crates"))?,
        );
        let name = self.name(names, input.name())?;
        let macro_root = self.macro_root(names)?;
        self.members.push(Member {
            name,
            root: module,
            macro_root,
            library: if word("no_core")? {
                Library::None
            } else if word("no_std")? {
                Library::Core
            } else {
                Library::Std
            },
            no_main: word("no_main")?,
            staged: false,
        });
        self.note_file(file, slot);
        Ok(Module {
            krate: slot,
            implicit_prelude: !word("no_implicit_prelude")?,
            parent: None,
            declaration: None,
            name,
            file,
            root,
        })
    }
    /// Record that the crate in `slot` requests `staged_api`.
    pub(super) fn mark_staged(&mut self, slot: CrateSlot) -> Result<(), Error> {
        self.members
            .get_mut(slot.index())
            .ok_or_else(|| Error::unlocated("guest crate slot is missing"))?
            .staged = true;
        Ok(())
    }
    fn note_file(&mut self, file: FileId, slot: CrateSlot) {
        let index = file.0 as usize;
        if index >= self.files.len() {
            self.files.resize(index + 1, CrateSlot::HOST);
        }
        self.files[index] = slot;
    }
}

impl Package {
    fn guest_state(&self) -> Option<&Guests> {
        self.source_core()?.guests.as_deref()
    }
    fn guest_state_mut(&mut self) -> Option<&mut Guests> {
        match self.source.as_deref_mut()? {
            Source::Core(core) => core.guests.as_deref_mut(),
            Source::Client(_) | Source::Staged(_) | Source::LocalMacros(_) => None,
        }
    }
    /// Keep the loaded guests in the session's source core.
    pub(super) fn install_guests(&mut self, guests: Guests) -> Result<(), Error> {
        let Some(Source::Core(core)) = self.source.as_deref_mut() else {
            return Err(Error::unlocated("guest crates require a source core"));
        };
        core.guests = Some(Box::new(guests));
        Ok(())
    }
    /// Whether this session resolves guest crates beside its own.
    #[must_use]
    pub fn has_guests(&self) -> bool {
        self.guest_state().is_some()
    }
    /// Every crate of a guest session, the host first; empty without guests.
    #[must_use]
    pub fn crate_members(&self) -> &[Member] {
        self.guest_state()
            .map_or(&[], |guests| guests.members.as_slice())
    }
    fn member(&self, slot: CrateSlot) -> Option<&Member> {
        self.guest_state()?.members.get(slot.index())
    }
    /// The module's crate slot identifies any guest that owns it.
    pub(crate) fn entered_guest(&self, module: ModuleId) -> Option<&Root> {
        let slot = self.modules.get(module.0 as usize)?.krate.guest()?;
        self.guest_state()?.roots.get(slot)
    }
    fn guest_definition(&self, guest: usize) -> Option<DefId> {
        self.guest_state()?
            .roots
            .get(guest)
            .map(|root| root.definition)
    }
    /// The root module of the crate that owns `module`: what `crate` names.
    #[must_use]
    pub fn crate_root(&self, module: ModuleId) -> ModuleId {
        let slot = self.modules[module.0 as usize].krate;
        if slot == CrateSlot::HOST {
            return ModuleId(0);
        }
        self.member(slot).map_or(ModuleId(0), |member| member.root)
    }
    /// The root module of the crate whose module tree loaded `file`.
    #[must_use]
    pub fn file_crate_root(&self, file: FileId) -> ModuleId {
        let slot = self.file_crate(file);
        if slot == CrateSlot::HOST {
            return ModuleId(0);
        }
        self.member(slot).map_or(ModuleId(0), |member| member.root)
    }
    /// Whether `key` is the root node of some guest crate.
    pub(crate) fn guest_root(&self, key: NodeKey) -> bool {
        self.crate_members().iter().skip(1).any(|member| {
            let module = &self.modules[member.root.0 as usize];
            module.file == key.file && module.root == key.node
        })
    }
    /// Whether `name` is a crate attribute that a guest root owns at `key`:
    /// its program selection reads it.
    pub(crate) fn guest_root_attribute(&self, key: NodeKey, name: &str) -> bool {
        name == "no_main" && self.guest_root(key)
    }
    /// The crate id that a macro defined in `module` gives its `$crate`.
    /// Zero, the using crate's own root, in every session without guests.
    pub(crate) fn macro_crate(&self, module: ModuleId) -> u32 {
        if !self.has_guests() {
            return 0;
        }
        self.modules[module.0 as usize].krate.0 + 1
    }
    fn macro_crate_slot(&self, name: NameId) -> Option<CrateSlot> {
        let position = self
            .crate_members()
            .iter()
            .position(|member| member.macro_root == name)?;
        u32::try_from(position).ok().map(CrateSlot)
    }
    /// The root module that a `$crate` path root names, for visibility.
    pub(super) fn macro_crate_module(&self, name: NameId) -> Option<ModuleId> {
        let slot = self.macro_crate_slot(name)?;
        self.member(slot).map(|member| member.root)
    }
    /// Each crate's root module and root node: the guests' in
    /// reverse driver order, then the host's.
    pub(crate) fn crate_roots(&self) -> impl Iterator<Item = (ModuleId, NodeKey)> + '_ {
        let guests = self.crate_members().iter().skip(1).rev();
        guests
            .map(|member| member.root)
            .chain([ModuleId(0)])
            .map(|module| {
                let root = &self.modules[module.0 as usize];
                let key = NodeKey {
                    file: root.file,
                    node: root.root,
                };
                (module, key)
            })
    }
    /// Whether the crate whose module tree loaded `file` requests
    /// `staged_api`; the session's request when it has no guests, whose
    /// joined request opens gates in every crate.
    #[must_use]
    pub(crate) fn staged_crate(&self, file: FileId) -> bool {
        match self.guest_state() {
            Some(guests) => guests
                .members
                .get(self.file_crate(file).index())
                .is_some_and(|member| member.staged),
            None => self
                .library_policy()
                .is_some_and(crate::resolve::stability::Library::is_staged),
        }
    }
    /// The crate whose module tree loaded `file`.
    #[must_use]
    pub fn file_crate(&self, file: FileId) -> CrateSlot {
        self.guest_state()
            .and_then(|guests| guests.files.get(file.0 as usize))
            .copied()
            .unwrap_or(CrateSlot::HOST)
    }
    /// The cfg scope and configuration of the crate whose module tree loaded
    /// `file`: a guest's own when the driver gave it cfg options, otherwise
    /// the session's.
    pub(crate) fn crate_config<'c>(
        &self,
        config: &'c crate::expand::config::Config,
        file: FileId,
    ) -> (usize, &'c crate::expand::config::Config) {
        let slot = self.file_crate(file);
        let guest = (slot != CrateSlot::HOST)
            .then(|| self.member(slot))
            .flatten()
            .map(|member| self.names.get(member.name));
        config.for_crate(guest)
    }
    /// Record that `file` was loaded for the crate that owns `module`.
    pub(super) fn note_file_crate(&mut self, file: FileId, module: ModuleId) {
        let slot = self.modules[module.0 as usize].krate;
        if let Some(guests) = self.guest_state_mut() {
            guests.note_file(file, slot);
        }
    }
    /// A guest crate's name and the directory of its root source, for a file
    /// that it loaded; nothing for the host's files.
    #[must_use]
    pub fn guest_origin(&self, file: FileId) -> Option<(&str, &std::path::Path)> {
        let member = self.member(self.file_crate(file))?;
        if member.root == ModuleId(0) {
            return None;
        }
        let root = self.modules[member.root.0 as usize].file;
        Some((
            self.names.get(member.name),
            self.file_paths[root.0 as usize].parent()?,
        ))
    }
    /// A source file's bring-up skip-list key: its path below the crate
    /// root's directory, or its whole path when it lies elsewhere. A guest
    /// crate's file is keyed `NAME:PATH`, its path below that guest's own root
    /// directory, which no host key can spell.
    #[must_use]
    pub fn skip_list_path(&self, file: FileId) -> String {
        let path = &self.file_paths[file.0 as usize];
        if let Some((guest, root)) = self.guest_origin(file) {
            let below = path.strip_prefix(root).unwrap_or(path);
            return format!("{guest}:{}", below.to_string_lossy());
        }
        self.file_paths
            .first()
            .and_then(|root| root.parent())
            .and_then(|root| path.strip_prefix(root).ok())
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned()
    }
    /// Whether the bring-up profile's skip list names this function item, so
    /// that a trapping stub replaces its body. Never in a strict compilation.
    #[must_use]
    pub fn bringup_listed(&self, item: NodeKey) -> bool {
        crate::support::bringup::selected().is_some_and(|stubs| {
            let start = self
                .unit(item.file)
                .configured
                .ast
                .node(item.node)
                .span
                .start;
            stubs.contains(&self.skip_list_path(item.file), start)
        })
    }
    /// The guest crates' slots in driver order; none in a strict session.
    pub(crate) fn guest_slots(&self) -> impl DoubleEndedIterator<Item = CrateSlot> + use<> {
        let count = u32::try_from(self.crate_members().len()).unwrap_or(u32::MAX);
        (1..count).map(CrateSlot)
    }
    /// The crate that supplies a guest session's entry point: its last guest.
    #[must_use]
    pub fn program_crate(&self) -> Option<&Member> {
        self.crate_members().last()
    }
    /// The crate name that `module_path!` spells for a crate's root module.
    pub(crate) fn root_name(&self, root: ModuleId) -> &str {
        if root == ModuleId(0) {
            return &self.crate_name;
        }
        self.names.get(self.modules[root.0 as usize].name)
    }
}

/// What resolution knows about one entered guest crate.
pub struct Root {
    /// The guest root's module definition.
    pub definition: DefId,
    /// The guest root's member scope, where `crate` paths start.
    pub scope: ScopeId,
    /// The guest's extern prelude: the host and the guests before it, then
    /// each alias that a root `extern crate` item adds.
    pub externs: ScopeId,
    /// Root scopes whose public macros form the guest's macro-use prelude;
    /// a later crate's macro replaces an earlier one's.
    macro_use: Vec<ScopeId>,
    /// The standard prelude module that the guest's attributes select.
    prelude: Option<(DefId, ScopeId)>,
}
impl Root {
    /// A public macro at the root of a crate this guest names `#[macro_use]`.
    fn macro_use(&self, facts: &Resolution, name: NameId) -> Option<(ScopeId, Binding)> {
        self.macro_use.iter().rev().find_map(|&scope| {
            facts
                .bindings
                .get(scope, name, Namespace::Macro)
                .filter(|binding| binding.access == Access::Public && !binding.ambiguous)
                .map(|binding| (scope, *binding))
        })
    }
    /// Whether this crate's selected prelude may supply a name before its
    /// macro-use or extern prelude. A macro lookup waits on
    /// each macro-use crate, whose exported macros may still be expanding.
    pub(super) fn prelude_permits(
        &self,
        facts: &Resolution,
        name: NameId,
        namespace: Namespace,
        observed: Option<&mut Observations>,
    ) -> bool {
        if namespace == Namespace::Macro
            && let Some(observed) = observed
        {
            for &scope in &self.macro_use {
                observed.dependency((scope, name, Namespace::Macro, ItemContext::ROOT));
            }
        }
        if namespace == Namespace::Macro && self.macro_use(facts, name).is_some()
            || namespace == Namespace::Type
                && facts.bindings.contains(self.externs, name, Namespace::Type)
        {
            return false;
        }
        true
    }
    /// The names found after every lexical scope and the standard prelude:
    /// the macro-use prelude, the primitive types, then the extern prelude.
    pub(super) fn terminal(
        &self,
        facts: &Resolution,
        name: NameId,
        namespace: Namespace,
        implicit_prelude: bool,
    ) -> Option<(ScopeId, Binding)> {
        if namespace == Namespace::Macro {
            return self.macro_use(facts, name);
        }
        if namespace != Namespace::Type {
            return None;
        }
        if let Some(&target) = facts.primitives.get(&name) {
            return Some((ScopeId(0), public(target)));
        }
        if !implicit_prelude {
            return None;
        }
        facts
            .bindings
            .get(self.externs, name, Namespace::Type)
            .map(|binding| (self.externs, *binding))
    }
    /// The standard prelude module, re-authenticated for a later phase.
    pub(crate) fn checked_prelude(
        &self,
        facts: &Resolution,
    ) -> Result<Option<ScopeId>, &'static str> {
        let Some((definition, scope)) = self.prelude else {
            return Ok(None);
        };
        if facts
            .definitions
            .get(definition.0 as usize)
            .is_none_or(|row| row.kind != DefKind::Module || row.members != Some(scope))
        {
            return Err("guest prelude module changed");
        }
        Ok(Some(scope))
    }
}
const fn public(target: DefId) -> Binding {
    Binding {
        target,
        access: Access::Public,
        origin: Origin::Direct,
        ambiguous: false,
    }
}
const fn prelude_module(edition: Edition) -> &'static str {
    match edition {
        Edition::Rust2015 => "rust_2015",
        Edition::Rust2018 => "rust_2018",
        Edition::Rust2021 => "rust_2021",
        Edition::Rust2024 => "rust_2024",
    }
}

impl Resolution {
    /// The definition of the crate root that a macro's `$crate` names.
    pub(super) fn macro_crate_root(&self, package: &Package, name: NameId) -> Option<DefId> {
        match package.macro_crate_slot(name)?.guest() {
            None => self.crate_root,
            Some(guest) => package.guest_definition(guest),
        }
    }
}

impl Resolver<'_> {
    /// The entered guest crate that owns `module`, if any.
    pub(super) fn guest(&self, module: ModuleId) -> Option<&Root> {
        self.package.entered_guest(module)
    }
    /// The scope where the `crate` keyword starts for a path in `module`.
    pub(super) fn crate_scope(&self, module: ModuleId) -> ScopeId {
        self.guest(module).map_or(ScopeId(0), |root| root.scope)
    }
    /// The definition of a crate root module: the host's or an entered guest's.
    pub(super) fn root_definition(&self, module: ModuleId) -> Option<DefId> {
        if module == ModuleId(0) {
            return self.facts.crate_root;
        }
        let root = self.guest(module)?;
        (self.facts.module_scopes.get(module.0 as usize) == Some(&root.scope))
            .then_some(root.definition)
    }
    /// Enter the next guest crate: create its root scope and definition, its
    /// extern prelude over the host and the earlier guests, and select the
    /// standard prelude and macro-use crate that its root attributes name.
    pub(super) fn enter_guest(&mut self, slot: CrateSlot) -> Result<(ModuleId, ScopeId), Error> {
        let entered = self
            .package
            .guest_state()
            .map_or(0, |guests| guests.roots.len());
        let (name, module, library) = self
            .package
            .member(slot)
            .filter(|_| slot.guest() == Some(entered))
            .map(|member| (member.name, member.root, member.library))
            .ok_or_else(|| Error::unlocated("guest crates are entered in driver order"))?;
        let file = self.package.modules[module.0 as usize].file;
        let scope = self.new_scope(None, module, true)?;
        self.facts
            .module_scopes
            .resize(self.package.modules.len(), ScopeId(0));
        self.facts.module_scopes[module.0 as usize] = scope;
        let definition = DefId(
            u32::try_from(self.facts.definitions.len())
                .map_err(|_| Error::at(file, 0, "too many definitions"))?,
        );
        self.facts.definitions.push(Definition {
            name,
            namespace: Namespace::Type,
            kind: DefKind::Module,
            scope,
            node: None,
            access: Access::Public,
            members: Some(scope),
        });
        let externs = self.new_scope(None, module, true)?;
        for earlier in 0..slot.0 {
            let earlier = CrateSlot(earlier);
            let target = match earlier.guest() {
                None => self.facts.crate_root,
                Some(guest) => self.package.guest_definition(guest),
            };
            let (Some(target), Some(member)) = (target, self.package.member(earlier)) else {
                return Err(Error::at(file, 0, "guest crate precedes its dependencies"));
            };
            self.facts
                .bindings
                .insert_key(
                    (externs, member.name, Namespace::Type, ItemContext::ROOT),
                    public(target),
                )
                .map_err(|message| Error::at(file, 0, message))?;
        }
        let mut root = Root {
            definition,
            scope,
            externs,
            macro_use: Vec::new(),
            prelude: None,
        };
        if let Some(library) = library.crate_name() {
            let (library_name, target, provider) =
                self.guest_library(file, name, externs, library)?;
            // Rust 2015 also introduces a private extern crate item at the root.
            if self.package.unit(file).edition() == Edition::Rust2015 {
                self.facts
                    .bindings
                    .insert_key(
                        (scope, library_name, Namespace::Type, ItemContext::ROOT),
                        Binding {
                            access: Access::Within(module),
                            ..public(target)
                        },
                    )
                    .map_err(|message| Error::at(file, 0, message))?;
            }
            root.prelude = Some(self.guest_prelude(file, name, provider, library)?);
            root.macro_use.push(provider);
        }
        self.package
            .guest_state_mut()
            .ok_or_else(|| Error::at(file, 0, "guest crates require a source core"))?
            .roots
            .push(root);
        Ok((module, scope))
    }
    /// The name, definition and scope of the standard library crate a guest's
    /// attributes select, which must be the host or an earlier guest.
    fn guest_library(
        &mut self,
        file: FileId,
        guest: NameId,
        externs: ScopeId,
        library: &str,
    ) -> Result<(NameId, DefId, ScopeId), Error> {
        let name = self
            .package
            .names
            .intern(library)
            .map_err(|message| Error::at(file, 0, message))?;
        self.facts
            .bindings
            .get(externs, name, Namespace::Type)
            .and_then(|binding| {
                self.facts.definitions[binding.target.0 as usize]
                    .members
                    .map(|scope| (name, binding.target, scope))
            })
            .ok_or_else(|| {
                Error::at(
                    file,
                    0,
                    format!(
                        "guest crate `{}` needs a `{library}` crate before it: none of the session's earlier crates has that name",
                        self.package.names.get(guest)
                    ),
                )
            })
    }
    /// `prelude::rust_EDITION` below a standard library root, both public.
    fn guest_prelude(
        &mut self,
        file: FileId,
        guest: NameId,
        provider: ScopeId,
        library: &str,
    ) -> Result<(DefId, ScopeId), Error> {
        let edition = prelude_module(self.package.unit(file).edition());
        let mut found = (DefId(0), provider);
        for segment in ["prelude", edition] {
            let name = self
                .package
                .names
                .intern(segment)
                .map_err(|message| Error::at(file, 0, message))?;
            found = self
                .facts
                .bindings
                .get(found.1, name, Namespace::Type)
                .filter(|binding| binding.access == Access::Public && !binding.ambiguous)
                .and_then(|binding| {
                    let row = &self.facts.definitions[binding.target.0 as usize];
                    (row.kind == DefKind::Module)
                        .then_some(row.members)
                        .flatten()
                        .map(|members| (binding.target, members))
                })
                .ok_or_else(|| {
                    Error::at(
                        file,
                        0,
                        format!(
                            "guest crate `{}` selects `{library}::prelude::{edition}`, which is not a public module declared before the guest",
                            self.package.names.get(guest)
                        ),
                    )
                })?;
        }
        Ok(found)
    }
    /// `#[macro_use] extern crate NAME;` at a guest's root adds that crate's
    /// public root macros to the guest's macro-use prelude. Returns the
    /// macros bound there now, whose waiting lookups must be retried.
    pub(super) fn guest_macro_use(
        &mut self,
        module: ModuleId,
        provider: ScopeId,
    ) -> Option<(ScopeId, Vec<NameId>)> {
        let slot = self.package.modules[module.0 as usize].krate.guest()?;
        let mut names: Vec<_> = self
            .facts
            .bindings
            .scope(provider)
            .filter(|&(_, namespace, _)| namespace == Namespace::Macro)
            .map(|(name, _, _)| name)
            .collect();
        names.sort_unstable_by_key(|name| name.0);
        let root = self.package.guest_state_mut()?.roots.get_mut(slot)?;
        if !root.macro_use.contains(&provider) {
            root.macro_use.push(provider);
        }
        Some((root.scope, names))
    }
}
