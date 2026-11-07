//! Exported declarative macros travel as canonical token images with typed rows.
//!
//! The image spells one delimited `macro_rules!` body. Two token-tree pieces are
//! adjacent exactly when they were adjacent in the definition source, and one
//! space separates them exactly when whitespace or comments did, so the image
//! keeps every token's jointness. The rows classify every piece. This module
//! only checks the structural agreement it can see without a lexer; a consumer
//! must re-lex the image once, rebuild the rows and recompile every rule.
use crate::support::edition::Edition;

/// Exported macros in one artifact.
pub const MAX_MACROS: usize = 4096;
/// Spelling bytes of one canonical body image.
pub const MAX_IMAGE_BYTES: usize = 1 << 20;
/// Typed rows of one canonical body image.
pub const MAX_ROWS: usize = 1 << 18;
/// Bytes of one exported macro name.
pub const MAX_NAME_BYTES: usize = 256;

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
        if name.is_empty()
            || name.len() > MAX_NAME_BYTES
            || !name
                .bytes()
                .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
            || name.as_bytes()[0].is_ascii_digit()
        {
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
        })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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
    }
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

/// Exported macros of one crate, sorted strictly by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Macros {
    rows: Vec<Macro>,
}
impl Macros {
    /// # Errors
    /// Rejects empty, oversized, unsorted and duplicate tables.
    pub fn new(rows: Vec<Macro>) -> Result<Self, &'static str> {
        if rows.is_empty() || rows.len() > MAX_MACROS {
            return Err("invalid exported macro count");
        }
        if rows.windows(2).any(|pair| pair[0].name >= pair[1].name) {
            return Err("exported macros must be unique and sorted by name");
        }
        Ok(Self { rows })
    }
    #[must_use]
    pub fn rows(&self) -> &[Macro] {
        &self.rows
    }
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Macro> {
        self.rows
            .binary_search_by(|row| row.name.as_str().cmp(name))
            .ok()
            .map(|index| &self.rows[index])
    }
    pub(super) fn bytes(&self) -> usize {
        self.rows
            .iter()
            .fold(0usize, |total, row| total.saturating_add(row.bytes()))
    }
}
