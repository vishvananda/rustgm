//! Module path policy over an injected source provider; no filesystem access here.
use super::{
    names::Names,
    package::{Error, Module, ModuleId, Package},
    unit::{FileId, NodeKey, Unit},
};
use crate::support::hash::HashMap;
use crate::{
    expand::{config::Config, configured::Effect},
    literal::Value,
    support::source::SourceFile,
    syntax::{
        ast::{Kind, MacroContext, NodeId},
        attribute::Input,
        item::ItemKind,
    },
};
use std::path::{Path, PathBuf};

pub struct InputFile {
    /// Canonical identity supplied by the driver, used only for cache/cycle keys.
    pub identity: PathBuf,
    /// Requested path controls module directories and observable source names.
    pub path: PathBuf,
    pub source: SourceFile,
}
pub trait Sources {
    /// # Errors
    /// Reports I/O errors; a nonexistent file is `Ok(None)`.
    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String>;
    /// Raw include resources retain every byte, including BOMs and CRLF.
    /// Binary-capable providers override this source-only default.
    ///
    /// # Errors
    /// Reports resource I/O errors, including invalid UTF-8 in the default provider.
    fn resource(&mut self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        self.read(path)
            .map(|file| file.map(|file| file.source.original().as_bytes().to_vec()))
    }
    /// Compiler libraries have no ambient environment. The driver supplies values.
    ///
    /// # Errors
    /// Reports inaccessible or non-Unicode values.
    fn environment(&mut self, _name: &str) -> Result<Option<String>, String> {
        Ok(None)
    }
}
struct Directory {
    path: PathBuf,
    relative: Option<String>,
}
struct Visit {
    module: ModuleId,
    node: NodeId,
    in_block: bool,
    owner: Option<NodeId>,
    include: Option<FileId>,
}
pub(crate) struct Loader<'a, S> {
    pub package: Package,
    sources: &'a mut S,
    config: &'a Config,
    directories: Vec<Directory>,
    identities: Vec<PathBuf>,
    cache: HashMap<PathBuf, FileId>,
    separate_instances: bool,
    includes: HashMap<(PathBuf, MacroContext), FileId>,
    include_requests: HashMap<(PathBuf, MacroContext), FileId>,
    resources: HashMap<PathBuf, Result<Option<Vec<u8>>, String>>,
    environment: HashMap<String, Result<Option<String>, String>>,
}
/// Load each source image once; disabled module declarations never request I/O.
///
/// # Errors
/// Reports malformed source, conflicting paths, missing modules and include cycles.
pub fn load(
    root: InputFile,
    sources: &mut impl Sources,
    config: &Config,
) -> Result<Package, Error> {
    load_with_context(root, sources, config, &super::bindings::Context::default())
}
/// Load with explicit source-provider ownership before any names are indexed.
/// # Errors
/// Reports ordinary loading errors and malformed source-provider configuration.
pub fn load_with_context(
    root: InputFile,
    sources: &mut impl Sources,
    config: &Config,
    context: &super::bindings::Context,
) -> Result<Package, Error> {
    let mut loader = Loader::with_context(root, sources, config, context)?;
    let root_node = loader.package.modules[0].root;
    let mut work = vec![Visit {
        module: ModuleId(0),
        node: root_node,
        in_block: false,
        owner: None,
        include: None,
    }];
    let mut children = Vec::new();
    while let Some(visit) = work.pop() {
        let module = &loader.package.modules[visit.module.0 as usize];
        let file = module.file;
        let unit = loader.package.unit(file);
        if !unit.configured.active[visit.node.get() as usize] {
            continue;
        }
        let kind = unit.configured.ast.node(visit.node).kind;
        if let Kind::Item(index) = kind
            && matches!(
                unit.configured.ast.items[index as usize].kind,
                ItemKind::Module { .. }
            )
            && visit.node != module.root
        {
            let module = loader.module(&visit, index)?;
            work.push(Visit {
                module,
                node: loader.package.modules[module.0 as usize].root,
                in_block: false,
                owner: None,
                include: None,
            });
            continue;
        }
        children.clear();
        unit.configured.ast.push_children(visit.node, &mut children);
        let owner = if matches!(kind, Kind::Attributed { .. }) {
            Some(visit.node)
        } else {
            None
        };
        let in_block = visit.in_block || matches!(kind, Kind::Block { .. });
        for node in children.iter().rev() {
            work.push(Visit {
                module: visit.module,
                node: *node,
                in_block,
                owner,
                include: None,
            });
        }
    }
    super::bindings::validate_features(&loader.package)?;
    Ok(loader.package)
}
impl<'a, S: Sources> Loader<'a, S> {
    pub(crate) fn with_context(
        root: InputFile,
        sources: &'a mut S,
        config: &'a Config,
        context: &super::bindings::Context,
    ) -> Result<Self, Error> {
        let mut names = Names::default();
        let name = names
            .intern("crate")
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        let unit = Unit::parse(root.source, config, &mut names)
            .map_err(|e| Error::at(FileId(0), e.offset, e.message))?;
        let root_node = unit.configured.ast.root;
        let no_std = unit
            .word_attribute(Some(root_node), "no_std")
            .map_err(|e| Error::at(FileId(0), e.offset, e.message))?;
        let prepared = super::bindings::policy::prepare(&unit, context)?;
        let source = prepared.source;
        let library = prepared.library;
        let implicit_prelude = !unit
            .word_attribute(Some(root_node), "no_implicit_prelude")
            .map_err(|e| Error::at(FileId(0), e.offset, e.message))?
            && source
                .as_deref()
                .is_none_or(|source| !source.owns_bindings());
        let mut loader = Self {
            package: Package {
                marks: crate::expand::marks::Marks::default(),
                source,
                library,
                macro_features: prepared.macros,
                runtime_checks: config.runtime_checks(),
                no_std,
                file_paths: vec![root.path.components().collect()],
                crate_name: if root.path == Path::new("<anon>") {
                    "rust_out".into()
                } else {
                    root.path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .replace('-', "_")
                },
                units: vec![unit],
                modules: vec![Module {
                    implicit_prelude,
                    parent: None,
                    declaration: None,
                    name,
                    file: FileId(0),
                    root: root_node,
                }],
                names,
                children: HashMap::default(),
            },
            sources,
            config,
            directories: vec![Directory {
                path: root.path.parent().unwrap_or(Path::new("")).into(),
                relative: None,
            }],
            identities: vec![root.identity.clone()],
            cache: HashMap::from_iter([(root.identity, FileId(0))]),
            separate_instances: false,
            includes: HashMap::default(),
            include_requests: HashMap::default(),
            resources: HashMap::default(),
            environment: HashMap::default(),
        };
        if loader.package.uses_source_bindings() {
            loader.package.crate_name = super::expansion::attributes::crate_name(
                &loader.package,
                context.crate_name.as_deref(),
            )?;
        }
        Ok(loader)
    }
    pub(crate) fn isolate_expansions(&mut self) {
        self.separate_instances = true;
    }
    fn cached_instance(&mut self, file: FileId, path: &Path) -> Result<FileId, Error> {
        if !self.separate_instances {
            return Ok(file);
        }
        let id = FileId(
            u32::try_from(self.package.units.len())
                .map_err(|_| Error::at(file, 0, "too many module instances"))?,
        );
        let unit = self
            .package
            .unit(file)
            .fresh_instance()
            .map_err(|e| Error::at(file, e.offset, e.message))?;
        self.identities
            .push(self.identities[file.0 as usize].clone());
        self.package.units.push(unit);
        self.package.file_paths.push(path.components().collect());
        Ok(id)
    }
    pub(crate) fn load_module(
        &mut self,
        module: ModuleId,
        node: NodeId,
        owner: Option<NodeId>,
        in_block: bool,
        include: Option<FileId>,
    ) -> Result<ModuleId, Error> {
        let file = self.package.modules[module.0 as usize].file;
        if let Some(id) = self.package.children.get(&(module, NodeKey { file, node })) {
            return Ok(*id);
        }
        let Kind::Item(index) = self.package.unit(file).configured.ast.node(node).kind else {
            return Err(Error::at(file, 0, "module declaration is not an item"));
        };
        self.module(
            &Visit {
                module,
                node,
                in_block,
                owner,
                include,
            },
            index,
        )
    }

    #[expect(
        clippy::too_many_lines,
        reason = "module directory ownership and file identity are one bounded transition"
    )]
    fn module(&mut self, visit: &Visit, index: u32) -> Result<ModuleId, Error> {
        let file = self.package.modules[visit.module.0 as usize].file;
        let item = self.package.unit(file).configured.ast.items[index as usize];
        let span = item
            .name
            .ok_or_else(|| Error::at(file, 0, "missing module name"))?;
        let name = self.package.name(file, span)?;
        let spelling = self.package.names.get(name).to_owned();
        let explicit = self.path(file, visit.owner)?;
        let included_directory = visit.include.map(|id| Directory {
            path: self.package.file_paths[id.0 as usize]
                .parent()
                .unwrap_or(Path::new(""))
                .into(),
            relative: None,
        });
        let directory = included_directory
            .as_ref()
            .unwrap_or(&self.directories[visit.module.0 as usize]);
        let mut path = directory.path.clone();
        let inline = matches!(item.kind, ItemKind::Module { items: Some(_) });
        if let Some(explicit) = &explicit {
            path.push(explicit);
        } else {
            if !inline && !spelling.is_ascii() {
                return Err(Error::at(
                    file,
                    span.start,
                    "non-ASCII file module requires #[path]",
                ));
            }
            if !inline && visit.in_block {
                return Err(Error::at(
                    file,
                    span.start,
                    "file module in a block requires #[path]",
                ));
            }
            if let Some(relative) = &directory.relative {
                path.push(relative);
            }
            path.push(&spelling);
        }
        let (source_file, root, directory) = if inline {
            (
                file,
                visit.node,
                Directory {
                    path,
                    relative: None,
                },
            )
        } else {
            let selected = if explicit.is_some() {
                self.sources.read(&path)
            } else {
                // Append instead of replacing an extension: identifiers may contain dots only via paths.
                let direct = path.with_extension("rs");
                let a = self
                    .sources
                    .read(&direct)
                    .map_err(|e| Error::at(file, span.start, e))?;
                let b = self
                    .sources
                    .read(&path.join("mod.rs"))
                    .map_err(|e| Error::at(file, span.start, e))?;
                if a.is_some() && b.is_some() {
                    return Err(Error::at(
                        file,
                        span.start,
                        "module has both name.rs and name/mod.rs",
                    ));
                }
                Ok(a.or(b))
            }
            .map_err(|e| Error::at(file, span.start, e))?
            .ok_or_else(|| Error::at(file, span.start, "module source file not found"))?;
            let mut ancestor = Some(visit.module);
            while let Some(id) = ancestor {
                let module = &self.package.modules[id.0 as usize];
                if self.identities[module.file.0 as usize] == selected.identity {
                    return Err(Error::at(
                        file,
                        span.start,
                        "circular module file inclusion",
                    ));
                }
                ancestor = module.parent;
            }
            let directory = Directory {
                path: selected.path.parent().unwrap_or(Path::new("")).into(),
                relative: if explicit.is_none()
                    && selected
                        .path
                        .file_name()
                        .is_some_and(|name| name != "mod.rs")
                {
                    Some(spelling)
                } else {
                    None
                },
            };
            let source_file = if let Some(id) = self.cache.get(&selected.identity).copied() {
                self.cached_instance(id, &selected.path)?
            } else {
                let id = FileId(
                    u32::try_from(self.package.units.len())
                        .map_err(|_| Error::at(file, span.start, "too many source files"))?,
                );
                let unit = Unit::parse(selected.source, self.config, &mut self.package.names)
                    .map_err(|e| Error::at(id, e.offset, e.message))?;
                self.cache.insert(selected.identity.clone(), id);
                self.identities.push(selected.identity);
                self.package.units.push(unit);
                self.package
                    .file_paths
                    .push(selected.path.components().collect());
                id
            };
            (
                source_file,
                self.package.unit(source_file).configured.ast.root,
                directory,
            )
        };
        let id = ModuleId(
            u32::try_from(self.package.modules.len())
                .map_err(|_| Error::at(file, span.start, "too many modules"))?,
        );
        let declaration = NodeKey {
            file,
            node: visit.node,
        };
        let implicit_prelude = self.package.modules[visit.module.0 as usize].implicit_prelude
            && !self
                .package
                .unit(file)
                .word_attribute(visit.owner, "no_implicit_prelude")
                .map_err(|e| Error::at(file, e.offset, e.message))?
            && !self
                .package
                .unit(source_file)
                .word_attribute(Some(root), "no_implicit_prelude")
                .map_err(|e| Error::at(source_file, e.offset, e.message))?;
        self.package.modules.push(Module {
            implicit_prelude,
            parent: Some(visit.module),
            declaration: Some(declaration),
            name,
            file: source_file,
            root,
        });
        self.package
            .children
            .insert((visit.module, declaration), id);
        self.directories.push(directory);
        Ok(id)
    }
    fn path(&self, file: FileId, owner: Option<NodeId>) -> Result<Option<String>, Error> {
        let unit = self.package.unit(file);
        let Some(owner) = owner
            .and_then(|id| unit.owner_indices[id.get() as usize])
            .map(|index| &unit.configured.owners[(index.get() - 1) as usize])
        else {
            return Ok(None);
        };
        for index in owner.attributes.clone() {
            let effect = &unit.configured.effects[index];
            let entry = effect.attribute;
            let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
            let attribute = entry.attribute;
            if entry.effect != Effect::Keep
                || attribute.name(forest, unit.source.text()) != Some("path")
            {
                continue;
            }
            if let Input::Equals(range) = attribute.input
                && range.end == range.start + 1
                && let Some(token) = forest.token(range.start)
                && let Some(Value::String(bytes)) =
                    crate::literal::interpret(unit.source.text(), token)
                        .map_err(|e| Error::at(file, e.offset, e.message))?
            {
                return String::from_utf8(bytes.into_owned())
                    .map(Some)
                    .map_err(|_| {
                        Error::at(file, attribute.span.start, "invalid UTF-8 module path")
                    });
            }
            return Err(Error::at(
                file,
                attribute.span.start,
                "path attribute requires a string literal",
            ));
        }
        Ok(None)
    }
}

impl<S: Sources> Loader<'_, S> {
    pub(crate) fn resource(&mut self, path: &Path) -> Result<Option<&[u8]>, String> {
        let value = self
            .resources
            .entry(path.components().collect())
            .or_insert_with(|| self.sources.resource(path));
        value
            .as_ref()
            .map(|bytes| bytes.as_deref())
            .map_err(Clone::clone)
    }
    pub(crate) fn environment(&mut self, name: &str) -> Result<Option<&str>, String> {
        let value = self
            .environment
            .entry(name.to_owned())
            .or_insert_with(|| self.sources.environment(name));
        value
            .as_ref()
            .map(|text| text.as_deref())
            .map_err(Clone::clone)
    }
}

impl<S: Sources> Loader<'_, S> {
    /// Cache parsed include fragments separately from complete module syntax.
    /// Requested aliases retain source names while sharing canonical parsed facts.
    pub(crate) fn include(
        &mut self,
        path: &Path,
        grammar: MacroContext,
    ) -> Result<(FileId, bool), Error> {
        let requested = (path.components().collect(), grammar);
        if let Some(file) = self.include_requests.get(&requested) {
            return Ok((*file, false));
        }
        let id = FileId(
            u32::try_from(self.package.units.len())
                .map_err(|_| Error::at(FileId(0), 0, "too many included source files"))?,
        );
        let input = self
            .sources
            .read(path)
            .map_err(|e| Error::at(id, 0, e))?
            .ok_or_else(|| {
                Error::at(
                    id,
                    0,
                    format!(
                        "included source {:?} does not exist",
                        path.to_string_lossy()
                    ),
                )
            })?;
        let key = (input.identity.clone(), grammar);
        let (unit, parsed) = if let Some(file) = self.includes.get(&key) {
            (self.package.unit(*file).fresh_instance(), false)
        } else {
            (
                Unit::parse_include(input.source, self.config, grammar, &mut self.package.names),
                true,
            )
        };
        let unit = unit.map_err(|e| Error::at(id, e.offset, e.message))?;
        self.package.units.push(unit);
        self.package
            .file_paths
            .push(input.path.components().collect());
        self.identities.push(input.identity);
        self.includes.entry(key).or_insert(id);
        self.include_requests.insert(requested, id);
        Ok((id, parsed))
    }
}
