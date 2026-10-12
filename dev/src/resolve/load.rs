//! Module path policy over an injected source provider; no filesystem access here.
use super::{
    names::Names,
    package::{Error, Module, ModuleId, Package},
    unit::{FileId, NodeKey, Unit},
};
use crate::{
    expand::{config::Config, configured::Effect},
    literal::Value,
    support::source::SourceFile,
    syntax::{
        ast::{Kind, NodeId},
        attribute::Input,
        item::ItemKind,
    },
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct InputFile {
    pub identity: PathBuf,
    pub source: SourceFile,
}
pub trait Sources {
    /// # Errors
    /// Reports I/O errors; a nonexistent file is `Ok(None)`.
    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String>;
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
}
pub(crate) struct Loader<'a, S> {
    pub package: Package,
    sources: &'a mut S,
    config: &'a Config,
    directories: Vec<Directory>,
    identities: Vec<PathBuf>,
    cache: HashMap<PathBuf, FileId>,
    separate_instances: bool,
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
    let mut loader = Loader::new(root, sources, config)?;
    let root_node = loader.package.modules[0].root;
    let mut work = vec![Visit {
        module: ModuleId(0),
        node: root_node,
        in_block: false,
        owner: None,
    }];
    let mut children = Vec::new();
    while let Some(visit) = work.pop() {
        let module = &loader.package.modules[visit.module.0 as usize];
        let file = module.file;
        let unit = loader.package.unit(file);
        if !unit.configured.active[visit.node.0 as usize] {
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
            });
        }
    }
    Ok(loader.package)
}
impl<'a, S: Sources> Loader<'a, S> {
    pub(crate) fn new(
        root: InputFile,
        sources: &'a mut S,
        config: &'a Config,
    ) -> Result<Self, Error> {
        let unit = Unit::parse(root.source, config)
            .map_err(|e| Error::at(FileId(0), e.offset, e.message))?;
        let mut names = Names::default();
        let name = names
            .intern("crate")
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        let root_node = unit.configured.ast.root;
        Ok(Self {
            package: Package {
                units: vec![unit],
                modules: vec![Module {
                    parent: None,
                    declaration: None,
                    name,
                    file: FileId(0),
                    root: root_node,
                }],
                names,
                children: HashMap::new(),
            },
            sources,
            config,
            directories: vec![Directory {
                path: root.identity.parent().unwrap_or(Path::new("")).into(),
                relative: None,
            }],
            identities: vec![root.identity.clone()],
            cache: HashMap::from([(root.identity, FileId(0))]),
            separate_instances: false,
        })
    }
    pub(crate) fn isolate_expansions(&mut self) {
        self.separate_instances = true;
    }
    fn cached_instance(&mut self, file: FileId) -> Result<FileId, Error> {
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
        Ok(id)
    }
    pub(crate) fn load_module(
        &mut self,
        module: ModuleId,
        node: NodeId,
        owner: Option<NodeId>,
        in_block: bool,
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
        let directory = &self.directories[visit.module.0 as usize];
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
                path: selected.identity.parent().unwrap_or(Path::new("")).into(),
                relative: if explicit.is_none()
                    && selected
                        .identity
                        .file_name()
                        .is_some_and(|name| name != "mod.rs")
                {
                    Some(spelling)
                } else {
                    None
                },
            };
            let source_file = if let Some(id) = self.cache.get(&selected.identity).copied() {
                self.cached_instance(id)?
            } else {
                let id = FileId(
                    u32::try_from(self.package.units.len())
                        .map_err(|_| Error::at(file, span.start, "too many source files"))?,
                );
                let unit = Unit::parse(selected.source, self.config)
                    .map_err(|e| Error::at(id, e.offset, e.message))?;
                self.cache.insert(selected.identity.clone(), id);
                self.identities.push(selected.identity);
                self.package.units.push(unit);
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
        self.package.modules.push(Module {
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
            .and_then(|id| unit.owner_indices[id.0 as usize])
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
