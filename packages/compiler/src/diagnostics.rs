/// The diagnostic collector: shared error/warning/info infrastructure.
///
/// All five phases write to a shared `DiagnosticCollector`. The collector
/// accumulates messages without halting compilation, enabling the compiler
/// to report as many issues as possible in a single run.
///
/// ## Code Ranges
///
/// | Phase    | Range         |
/// |----------|---------------|
/// | PARSE    | URD100–URD199 |
/// | IMPORT   | URD200–URD299 |
/// | LINK     | URD300–URD399 |
/// | VALIDATE | URD400–URD499 |
/// | EMIT     | URD500–URD599 |
/// | ANALYZE (FactSet) | URD600–URD699 |

use crate::span::Span;

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
    pub related: Vec<RelatedInfo>,
}

/// Additional context for a diagnostic (e.g. "first declared here").
#[derive(Debug, Clone)]
pub struct RelatedInfo {
    pub message: String,
    pub span: Span,
}

/// Accumulates diagnostics from all compiler phases.
#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an error diagnostic.
    pub fn error(&mut self, code: impl Into<String>, message: impl Into<String>, span: Span) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span,
            suggestion: None,
            related: Vec::new(),
        });
    }

    /// Record a warning diagnostic.
    pub fn warning(&mut self, code: impl Into<String>, message: impl Into<String>, span: Span) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            span,
            suggestion: None,
            related: Vec::new(),
        });
    }

    /// Record an info diagnostic.
    pub fn info(&mut self, code: impl Into<String>, message: impl Into<String>, span: Span) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Info,
            code: code.into(),
            message: message.into(),
            span,
            suggestion: None,
            related: Vec::new(),
        });
    }

    /// Record a fully specified diagnostic.
    pub fn emit(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Returns `true` if any Error-severity diagnostic has been recorded.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    /// Returns all diagnostics, sorted in deterministic order:
    /// 1. By file path (alphabetical)
    /// 2. By source position (line, then column)
    /// 3. By severity (errors before warnings before info)
    pub fn sorted(&self) -> Vec<&Diagnostic> {
        let mut sorted: Vec<&Diagnostic> = self.diagnostics.iter().collect();
        sorted.sort_by(|a, b| {
            a.span.file.cmp(&b.span.file)
                .then(a.span.start_line.cmp(&b.span.start_line))
                .then(a.span.start_col.cmp(&b.span.start_col))
                .then(a.severity.cmp(&b.severity))
        });
        sorted
    }

    /// Returns the total number of diagnostics.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns `true` if no diagnostics have been recorded.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns all diagnostics as a slice.
    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
