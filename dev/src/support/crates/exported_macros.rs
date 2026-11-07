//! Exported declarative macros travel as canonical token images with typed rows.
//!
//! The image spells one delimited `macro_rules!` body. Two token-tree pieces are
//! adjacent exactly when they were adjacent in the definition source, and one
//! space separates them exactly when whitespace or comments did, so the image
//! keeps every token's jointness. The rows classify every piece. This module
//! only checks the structural agreement it can see without a lexer; a consumer
//! must re-lex the image once, rebuild the rows and recompile every rule.
//!
//! Version two adds `#[rustc_builtin_macro]` rows, which carry no image but the
//! expander identity, and typed `allow_internal_unstable` feature lists on
//! declarative rows. Every row has a module placement; names are unique per
//! placement. Re-export rows bind a further placement and name to one
//! definition row of this crate or of a dependency, by that row's identity.
//!
//! Version three adds `allow_internal_unstable` lists to builtin rows and the
//! `#[unstable]` feature of any definition row; a consumer must enable that
//! feature, or call the row from an expansion allowed to use it.
use crate::support::edition::Edition;

/// Exported macros in one artifact.
pub const MAX_MACROS: usize = 4096;
/// Spelling bytes of one canonical body image.
pub const MAX_IMAGE_BYTES: usize = 1 << 20;
/// Typed rows of one canonical body image.
pub const MAX_ROWS: usize = 1 << 18;
/// Bytes of one exported macro name.
pub const MAX_NAME_BYTES: usize = 256;
/// Bytes of one builtin expander identity spelling.
pub const MAX_IDENTITY_BYTES: usize = 64;
/// Components of one module placement.
pub const MAX_PLACEMENT_DEPTH: usize = 64;
/// Features of one `allow_internal_unstable` list, or helpers of one builtin derive.
pub const MAX_FEATURES: usize = 64;

/// The declaration syntax of a builtin row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syntax {
    MacroRules,
    Macro,
}
impl Syntax {
    pub const ALL: [Self; 2] = [Self::MacroRules, Self::Macro];
}

/// How a builtin row is invoked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacroKind {
    Bang,
    Attribute,
    Derive,
}
impl MacroKind {
    pub const ALL: [Self; 3] = [Self::Bang, Self::Attribute, Self::Derive];
}

/// ASCII identifiers: macro names, builtin identities, features and helpers.
fn valid_identifier(name: &str, limit: usize) -> bool {
    !name.is_empty()
        && name.len() <= limit
        && name
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
        && !name.as_bytes()[0].is_ascii_digit()
}

/// A bounded list of identifiers, strictly sorted so that its spelling is canonical.
fn validate_names(names: &[String], message: &'static str) -> Result<(), &'static str> {
    if names.len() > MAX_FEATURES
        || names
            .iter()
            .any(|name| !valid_identifier(name, MAX_NAME_BYTES))
        || names.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(message);
    }
    Ok(())
}

fn validate_unstable(feature: Option<&String>) -> Result<(), &'static str> {
    if feature.is_some_and(|feature| !valid_identifier(feature, MAX_NAME_BYTES)) {
        return Err("invalid exported macro unstable feature");
    }
    Ok(())
}

fn names_bytes(names: &[String]) -> usize {
    names
        .iter()
        .fold(0usize, |total, name| total.saturating_add(name.len()))
}

/// The metadata class of one non-delimiter token. Punctuation identity is its spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenClass {
    Identifier,
    RawIdentifier,
    Lifetime,
    RawLifetime,
    Integer,
    Float,
    Character,
    Byte,
    String,
    ByteString,
    CString,
    RawString,
    RawByteString,
    RawCString,
    Punctuation,
}
impl TokenClass {
    pub const ALL: [Self; 15] = [
        Self::Identifier,
        Self::RawIdentifier,
        Self::Lifetime,
        Self::RawLifetime,
        Self::Integer,
        Self::Float,
        Self::Character,
        Self::Byte,
        Self::String,
        Self::ByteString,
        Self::CString,
        Self::RawString,
        Self::RawByteString,
        Self::RawCString,
        Self::Punctuation,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delimiter {
    Parenthesis,
    Bracket,
    Brace,
}
impl Delimiter {
    pub const ALL: [Self; 3] = [Self::Parenthesis, Self::Bracket, Self::Brace];
    #[must_use]
    pub const fn open(self) -> u8 {
        match self {
            Self::Parenthesis => b'(',
            Self::Bracket => b'[',
            Self::Brace => b'{',
        }
    }
    #[must_use]
    pub const fn close(self) -> u8 {
        match self {
            Self::Parenthesis => b')',
            Self::Bracket => b']',
            Self::Brace => b'}',
        }
    }
}

/// One preorder token-tree row. A group counts all of its descendant rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Token { class: TokenClass, length: u32 },
    Group { delimiter: Delimiter, rows: u32 },
}

/// Builds the canonical image and its rows from one preorder token-tree walk.
/// Every piece (token, open or close delimiter) directly follows the previous
/// one, or follows it after exactly one space when its caller says the source
/// separated them. The first piece is never separated.
#[derive(Default)]
pub struct ImageBuilder {
    image: String,
    rows: Vec<Row>,
    open: Vec<(usize, Delimiter)>,
}
impl ImageBuilder {
    fn separate(&mut self, separated: bool) -> Result<(), &'static str> {
        if separated {
            if self.image.is_empty() {
                return Err("exported macro image cannot begin with a separator");
            }
            self.image.push(' ');
        }
        Ok(())
    }
    /// Separates one new row and admits it under the budget.
    fn admit(&mut self, separated: bool) -> Result<(), &'static str> {
        self.separate(separated)?;
        if self.image.len() >= MAX_IMAGE_BYTES || self.rows.len() >= MAX_ROWS {
            return Err("exported macro body exceeds the token metadata budget");
        }
        Ok(())
    }
    /// # Errors
    /// Rejects empty spellings, a leading separator and budget overflow.
    pub fn token(
        &mut self,
        class: TokenClass,
        spelling: &str,
        separated: bool,
    ) -> Result<(), &'static str> {
        self.admit(separated)?;
        let length = u32::try_from(spelling.len())
            .map_err(|_| "exported macro body exceeds the token metadata budget")?;
        if length == 0 {
            return Err("exported macro token has no spelling");
        }
        self.image.push_str(spelling);
        self.rows.push(Row::Token { class, length });
        Ok(())
    }
    /// # Errors
    /// Rejects a leading separator and budget overflow.
    pub fn open(&mut self, delimiter: Delimiter, separated: bool) -> Result<(), &'static str> {
        self.admit(separated)?;
        self.image.push(char::from(delimiter.open()));
        self.open.push((self.rows.len(), delimiter));
        self.rows.push(Row::Group { delimiter, rows: 0 });
        Ok(())
    }
    /// # Errors
    /// Rejects an unbalanced close.
    pub fn close(&mut self, separated: bool) -> Result<(), &'static str> {
        let (start, delimiter) = self
            .open
            .pop()
            .ok_or("unbalanced exported macro token tree")?;
        let rows = u32::try_from(self.rows.len() - start - 1)
            .map_err(|_| "exported macro body exceeds the token metadata budget")?;
        self.rows[start] = Row::Group { delimiter, rows };
        self.separate(separated)?;
        self.image.push(char::from(delimiter.close()));
        Ok(())
    }
    /// # Errors
    /// Rejects unbalanced trees.
    pub fn finish(self) -> Result<(String, Vec<Row>), &'static str> {
        if !self.open.is_empty() {
            return Err("unbalanced exported macro token tree");
        }
        Ok((self.image, self.rows))
    }
}

/// One `#[macro_export] macro_rules!` definition, published at the crate root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Macro {
    name: String,
    edition: Edition,
    local_inner_macros: bool,
    rules: u32,
    image: String,
    rows: Vec<Row>,
    internal_unstable: Vec<String>,
    unstable: Option<String>,
}
impl Macro {
    /// # Errors
    /// Rejects names, rule counts and rows that disagree with the image layout.
    pub fn new(
        name: String,
        edition: Edition,
        local_inner_macros: bool,
        rules: u32,
        image: String,
        rows: Vec<Row>,
    ) -> Result<Self, &'static str> {
        if !valid_identifier(&name, MAX_NAME_BYTES) {
            return Err("invalid exported macro name");
        }
        if rules == 0 {
            return Err("exported macro has no rules");
        }
        if image.len() > MAX_IMAGE_BYTES || rows.len() > MAX_ROWS {
            return Err("exported macro body exceeds the token metadata budget");
        }
        validate_layout(image.as_bytes(), &rows)?;
        Ok(Self {
            name,
            edition,
            local_inner_macros,
            rules,
            image,
            rows,
            internal_unstable: Vec::new(),
            unstable: None,
        })
    }
    /// Attach the definition's typed `allow_internal_unstable` features.
    /// # Errors
    /// Rejects lists that are oversized, unsorted, duplicated or not identifiers.
    pub fn with_internal_unstable(mut self, features: Vec<String>) -> Result<Self, &'static str> {
        validate_names(
            &features,
            "invalid exported macro internal-unstable features",
        )?;
        self.internal_unstable = features;
        Ok(self)
    }
    /// Attach the definition's `#[unstable]` feature, if any.
    /// # Errors
    /// Rejects a feature that is not an identifier.
    pub fn with_unstable(mut self, feature: Option<String>) -> Result<Self, &'static str> {
        validate_unstable(feature.as_ref())?;
        self.unstable = feature;
        Ok(self)
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// The feature a consumer needs to call this definition.
    #[must_use]
    pub fn unstable(&self) -> Option<&str> {
        self.unstable.as_deref()
    }
    /// Features an expansion of this definition may use, sorted and unique.
    #[must_use]
    pub fn internal_unstable(&self) -> &[String] {
        &self.internal_unstable
    }
    #[must_use]
    pub const fn edition(&self) -> Edition {
        self.edition
    }
    #[must_use]
    pub const fn local_inner_macros(&self) -> bool {
        self.local_inner_macros
    }
    #[must_use]
    pub const fn rules(&self) -> u32 {
        self.rules
    }
    #[must_use]
    pub fn image(&self) -> &str {
        &self.image
    }
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub(super) fn bytes(&self) -> usize {
        self.name
            .len()
            .saturating_add(self.image.len())
            .saturating_add(self.rows.len().saturating_mul(size_of::<Row>()))
            .saturating_add(names_bytes(&self.internal_unstable))
            .saturating_add(self.unstable.as_ref().map_or(0, String::len))
    }
}

/// One `#[rustc_builtin_macro]` export. It carries no image: a consumer binds
/// its own expander of this identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltinMacro {
    placement: Vec<String>,
    name: String,
    syntax: Syntax,
    identity: String,
    kind: MacroKind,
    helpers: Vec<String>,
    internal_unstable: Vec<String>,
    unstable: Option<String>,
}
impl BuiltinMacro {
    /// # Errors
    /// Rejects invalid names, identities, placements and helper lists, a
    /// `macro_rules!` row outside the crate root and helpers on a non-derive.
    pub fn new(
        placement: Vec<String>,
        name: String,
        syntax: Syntax,
        identity: String,
        kind: MacroKind,
        helpers: Vec<String>,
    ) -> Result<Self, &'static str> {
        if !valid_identifier(&name, MAX_NAME_BYTES) {
            return Err("invalid exported macro name");
        }
        if !valid_identifier(&identity, MAX_IDENTITY_BYTES) {
            return Err("invalid exported builtin macro identity");
        }
        validate_placement(&placement)?;
        if syntax == Syntax::MacroRules && !placement.is_empty() {
            return Err("exported builtin macro_rules rows belong to the crate root");
        }
        validate_names(&helpers, "invalid exported builtin derive helpers")?;
        if !helpers.is_empty() && kind != MacroKind::Derive {
            return Err("only builtin derive rows declare helper attributes");
        }
        Ok(Self {
            placement,
            name,
            syntax,
            identity,
            kind,
            helpers,
            internal_unstable: Vec::new(),
            unstable: None,
        })
    }
    /// Attach the definition's `allow_internal_unstable` features.
    /// # Errors
    /// Rejects lists that are oversized, unsorted, duplicated or not identifiers.
    pub fn with_internal_unstable(mut self, features: Vec<String>) -> Result<Self, &'static str> {
        validate_names(
            &features,
            "invalid exported macro internal-unstable features",
        )?;
        self.internal_unstable = features;
        Ok(self)
    }
    /// Attach the definition's `#[unstable]` feature, if any.
    /// # Errors
    /// Rejects a feature that is not an identifier.
    pub fn with_unstable(mut self, feature: Option<String>) -> Result<Self, &'static str> {
        validate_unstable(feature.as_ref())?;
        self.unstable = feature;
        Ok(self)
    }
    /// Features an expansion of this row may use, sorted and unique.
    #[must_use]
    pub fn internal_unstable(&self) -> &[String] {
        &self.internal_unstable
    }
    /// The feature a consumer needs to call this row.
    #[must_use]
    pub fn unstable(&self) -> Option<&str> {
        self.unstable.as_deref()
    }
    /// The module path of the scope that binds this row; empty is the crate root.
    #[must_use]
    pub fn placement(&self) -> &[String] {
        &self.placement
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn syntax(&self) -> Syntax {
        self.syntax
    }
    /// The expander that `#[rustc_builtin_macro]` or `#[rustc_builtin_macro(name)]` selects.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
    #[must_use]
    pub const fn kind(&self) -> MacroKind {
        self.kind
    }
    #[must_use]
    pub fn helpers(&self) -> &[String] {
        &self.helpers
    }
    fn key(&self) -> (&[String], &str) {
        (&self.placement, &self.name)
    }
    fn bytes(&self) -> usize {
        self.name
            .len()
            .saturating_add(self.identity.len())
            .saturating_add(names_bytes(&self.placement))
            .saturating_add(names_bytes(&self.helpers))
            .saturating_add(names_bytes(&self.internal_unstable))
            .saturating_add(self.unstable.as_ref().map_or(0, String::len))
    }
}

fn validate_placement(placement: &[String]) -> Result<(), &'static str> {
    if placement.len() > MAX_PLACEMENT_DEPTH {
        return Err("invalid exported macro placement");
    }
    for component in placement {
        super::identity::validate_component(component)?;
    }
    Ok(())
}

/// One more public binding of a definition row: this crate's own row when the
/// owner is 0, otherwise a row of dependency `owner - 1`. The target is that
/// row's identity, its placement and name in its owner's table, never a path
/// a consumer resolves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReexportMacro {
    placement: Vec<String>,
    name: String,
    owner: u32,
    target_placement: Vec<String>,
    target_name: String,
}
impl ReexportMacro {
    /// # Errors
    /// Rejects invalid names and placements of the binding or its target.
    pub fn new(
        placement: Vec<String>,
        name: String,
        owner: u32,
        target_placement: Vec<String>,
        target_name: String,
    ) -> Result<Self, &'static str> {
        if !valid_identifier(&name, MAX_NAME_BYTES)
            || !valid_identifier(&target_name, MAX_NAME_BYTES)
        {
            return Err("invalid exported macro name");
        }
        validate_placement(&placement)?;
        validate_placement(&target_placement)?;
        Ok(Self {
            placement,
            name,
            owner,
            target_placement,
            target_name,
        })
    }
    /// The module path of the scope that binds this re-export.
    #[must_use]
    pub fn placement(&self) -> &[String] {
        &self.placement
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// 0 for this artifact, otherwise one more than the dependency's index.
    #[must_use]
    pub const fn owner(&self) -> u32 {
        self.owner
    }
    /// The placement of the definition row in its owner's table.
    #[must_use]
    pub fn target_placement(&self) -> &[String] {
        &self.target_placement
    }
    #[must_use]
    pub fn target_name(&self) -> &str {
        &self.target_name
    }
    fn key(&self) -> (&[String], &str) {
        (&self.placement, &self.name)
    }
    fn bytes(&self) -> usize {
        self.name
            .len()
            .saturating_add(self.target_name.len())
            .saturating_add(names_bytes(&self.placement))
            .saturating_add(names_bytes(&self.target_placement))
    }
}

/// Which definition row of one table a placement and name identify.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DefinitionRow {
    Declarative(usize),
    Builtin(usize),
}

/// The body is one root group laid out as [`ImageBuilder`] spells it: pieces
/// directly adjacent or separated by exactly one space.
fn validate_layout(image: &[u8], rows: &[Row]) -> Result<(), &'static str> {
    const MALFORMED: &str = "exported macro rows disagree with their canonical image";
    match rows.first() {
        Some(Row::Group { rows: body, .. }) if *body as usize + 1 == rows.len() => {}
        _ => return Err("exported macro body requires one delimited group"),
    }
    let mut cursor = 0usize;
    let expect = |cursor: &mut usize, byte: u8| {
        let found = image.get(*cursor) == Some(&byte);
        *cursor += 1;
        found.then_some(()).ok_or(MALFORMED)
    };
    // Every piece but the first may follow one separating space. No piece
    // begins with a space, so the layout stays unambiguous.
    let separator = |cursor: &mut usize| {
        if image.get(*cursor) == Some(&b' ') {
            *cursor += 1;
        }
    };
    // Each open group keeps its delimiter and exclusive end row.
    let mut open: Vec<(Delimiter, usize)> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            separator(&mut cursor);
        }
        match *row {
            Row::Token { length, .. } => {
                // Literals may contain spaces; no token begins or ends with one.
                let end = cursor.saturating_add(length as usize);
                match image.get(cursor..end) {
                    Some([first, .., last]) if *first != b' ' && *last != b' ' => {}
                    Some([only]) if *only != b' ' => {}
                    _ => return Err(MALFORMED),
                }
                cursor = end;
            }
            Row::Group {
                delimiter,
                rows: count,
            } => {
                expect(&mut cursor, delimiter.open())?;
                let end = index.checked_add(count as usize + 1).ok_or(MALFORMED)?;
                if open.last().is_some_and(|&(_, outer)| end > outer) {
                    return Err(MALFORMED);
                }
                open.push((delimiter, end));
            }
        }
        while let Some(&(delimiter, end)) = open.last() {
            if end != index + 1 {
                break;
            }
            open.pop();
            separator(&mut cursor);
            expect(&mut cursor, delimiter.close())?;
        }
    }
    (open.is_empty() && cursor == image.len())
        .then_some(())
        .ok_or(MALFORMED)
}

/// Exported macros of one crate. Declarative rows sit at the crate root,
/// sorted strictly by name; builtin and re-export rows are each sorted
/// strictly by placement and name. No scope binds one name twice, and every
/// re-export of this crate's own row names a definition row of this table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Macros {
    rows: Vec<Macro>,
    builtins: Vec<BuiltinMacro>,
    reexports: Vec<ReexportMacro>,
}
impl Macros {
    /// # Errors
    /// Rejects empty, oversized, unsorted and duplicate tables.
    pub fn new(rows: Vec<Macro>) -> Result<Self, &'static str> {
        Self::with_builtins(rows, Vec::new())
    }
    /// # Errors
    /// Rejects empty, oversized, unsorted and duplicate tables, and one name
    /// bound twice in a scope.
    pub fn with_builtins(
        rows: Vec<Macro>,
        builtins: Vec<BuiltinMacro>,
    ) -> Result<Self, &'static str> {
        Self::with_reexports(rows, builtins, Vec::new())
    }
    /// # Errors
    /// Rejects empty, oversized, unsorted and duplicate tables, one name bound
    /// twice in a scope, and a local re-export whose target is not a
    /// definition row of this table.
    pub fn with_reexports(
        rows: Vec<Macro>,
        builtins: Vec<BuiltinMacro>,
        reexports: Vec<ReexportMacro>,
    ) -> Result<Self, &'static str> {
        let count = rows
            .len()
            .saturating_add(builtins.len())
            .saturating_add(reexports.len());
        if count == 0 || count > MAX_MACROS {
            return Err("invalid exported macro count");
        }
        if rows.windows(2).any(|pair| pair[0].name >= pair[1].name)
            || builtins
                .windows(2)
                .any(|pair| pair[0].key() >= pair[1].key())
            || reexports
                .windows(2)
                .any(|pair| pair[0].key() >= pair[1].key())
        {
            return Err("exported macros must be unique and sorted by name");
        }
        let mut keys: Vec<(&[String], &str)> = rows
            .iter()
            .map(|row| (&[][..], row.name()))
            .chain(builtins.iter().map(BuiltinMacro::key))
            .chain(reexports.iter().map(ReexportMacro::key))
            .collect();
        keys.sort_unstable();
        if keys.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err("duplicate exported macro name in one scope");
        }
        let table = Self {
            rows,
            builtins,
            reexports,
        };
        if table.reexports.iter().any(|row| {
            row.owner == 0
                && table
                    .definition(&row.target_placement, &row.target_name)
                    .is_none()
        }) {
            return Err("exported macro re-export names no definition row of its crate");
        }
        Ok(table)
    }
    /// The declarative root rows.
    #[must_use]
    pub fn rows(&self) -> &[Macro] {
        &self.rows
    }
    #[must_use]
    pub fn builtins(&self) -> &[BuiltinMacro] {
        &self.builtins
    }
    #[must_use]
    pub fn reexports(&self) -> &[ReexportMacro] {
        &self.reexports
    }
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Macro> {
        self.rows
            .binary_search_by(|row| row.name.as_str().cmp(name))
            .ok()
            .map(|index| &self.rows[index])
    }
    /// The definition row this table publishes at one placement and name. A
    /// re-export row is never a definition.
    #[must_use]
    pub fn definition(&self, placement: &[String], name: &str) -> Option<DefinitionRow> {
        if placement.is_empty()
            && let Ok(index) = self
                .rows
                .binary_search_by(|row| row.name.as_str().cmp(name))
        {
            return Some(DefinitionRow::Declarative(index));
        }
        self.builtins
            .binary_search_by(|row| row.key().cmp(&(placement, name)))
            .ok()
            .map(DefinitionRow::Builtin)
    }
    /// The `allow_internal_unstable` features of one definition row.
    #[must_use]
    pub fn internal_unstable(&self, row: DefinitionRow) -> Option<&[String]> {
        match row {
            DefinitionRow::Declarative(index) => self.rows.get(index).map(Macro::internal_unstable),
            DefinitionRow::Builtin(index) => self
                .builtins
                .get(index)
                .map(BuiltinMacro::internal_unstable),
        }
    }
    /// The `#[unstable]` feature of one definition row; the outer `None` is
    /// an unknown row.
    #[must_use]
    pub fn unstable(&self, row: DefinitionRow) -> Option<Option<&str>> {
        match row {
            DefinitionRow::Declarative(index) => self.rows.get(index).map(Macro::unstable),
            DefinitionRow::Builtin(index) => self.builtins.get(index).map(BuiltinMacro::unstable),
        }
    }
    /// The largest dependency ordinal a re-export row names.
    #[must_use]
    pub fn max_owner(&self) -> u32 {
        self.reexports
            .iter()
            .map(ReexportMacro::owner)
            .max()
            .unwrap_or(0)
    }
    /// The least envelope version that holds every row: 3 for a builtin
    /// allowance list or any `#[unstable]` row, 2 for a builtin row, a
    /// re-export row or a declarative allowance list, and otherwise 1, whose
    /// bytes stay exactly as before.
    #[must_use]
    pub fn version(&self) -> u8 {
        if self.rows.iter().any(|row| row.unstable.is_some())
            || self
                .builtins
                .iter()
                .any(|row| row.unstable.is_some() || !row.internal_unstable.is_empty())
        {
            3
        } else if !self.builtins.is_empty()
            || !self.reexports.is_empty()
            || self
                .rows
                .iter()
                .any(|row| !row.internal_unstable.is_empty())
        {
            2
        } else {
            1
        }
    }
    /// Rows plus placement components, for the shared component budget.
    pub(super) fn atoms(&self) -> usize {
        let builtins = self.builtins.iter().fold(self.rows.len(), |total, row| {
            total.saturating_add(1 + row.placement.len())
        });
        self.reexports.iter().fold(builtins, |total, row| {
            total.saturating_add(1 + row.placement.len() + row.target_placement.len())
        })
    }
    pub(super) fn bytes(&self) -> usize {
        let declarative = self
            .rows
            .iter()
            .fold(0usize, |total, row| total.saturating_add(row.bytes()));
        let builtins = self
            .builtins
            .iter()
            .fold(declarative, |total, row| total.saturating_add(row.bytes()));
        self.reexports
            .iter()
            .fold(builtins, |total, row| total.saturating_add(row.bytes()))
    }
}
