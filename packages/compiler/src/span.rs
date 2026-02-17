/// Source position tracking for diagnostics and source maps.
///
/// Every AST node carries a `Span` recording its exact position in source.
/// Lines and columns are 1-indexed. Columns are byte offsets within the line.

/// A file path normalised to forward slashes, relative to the entry file's directory.
pub type FilePath = String;

/// A source span: file path + start/end positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub file: FilePath,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    pub fn new(file: FilePath, start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Self {
        Self { file, start_line, start_col, end_line, end_col }
    }

    /// A synthetic span for compiler-generated constructs.
    pub fn synthetic() -> Self {
        Self { file: String::new(), start_line: 0, start_col: 0, end_line: 0, end_col: 0 }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.start_line, self.start_col)
    }
}
