//! Symbol types extracted from source code.

use serde::{Deserialize, Serialize};

/// A symbol extracted from source code via tree-sitter.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    /// Fully qualified name (e.g., "golem_core::id::GolemId").
    pub name: String,
    /// Symbol kind.
    pub kind: SymbolKind,
    /// File path relative to project root.
    pub file: String,
    /// Line number (1-indexed).
    pub line: u32,
    /// The signature text (function signature, type definition, etc.).
    pub signature: String,
    /// Visibility.
    pub visibility: Visibility,
    /// Doc comment (if any).
    pub doc: Option<String>,
    /// Blake3 hash of the signature for change detection.
    pub content_hash: [u8; 32],
}

/// What kind of symbol this is.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    /// A function or method.
    Function,
    /// A struct.
    Struct,
    /// An enum.
    Enum,
    /// A trait.
    Trait,
    /// A type alias.
    TypeAlias,
    /// A const or static.
    Const,
    /// A module declaration.
    Module,
    /// An impl block.
    Impl,
    /// A use/import statement.
    Use,
    /// A macro definition.
    Macro,
}

/// Symbol visibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// `pub`
    Public,
    /// `pub(crate)`
    Crate,
    /// `pub(super)` or `pub(in path)`
    Restricted,
    /// Private (no visibility modifier).
    Private,
}

/// A reference from one symbol to another (import, call, type usage).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolRef {
    /// The file containing the reference.
    pub from_file: String,
    /// Line of the reference.
    pub from_line: u32,
    /// The symbol being referenced (name or path).
    pub target: String,
    /// Kind of reference.
    pub ref_kind: RefKind,
}

/// What kind of reference this is.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefKind {
    /// `use` import statement.
    Import,
    /// Type usage in a signature.
    TypeRef,
    /// Function call.
    Call,
    /// Trait implementation.
    ImplTrait,
}
