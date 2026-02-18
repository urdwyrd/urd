/// Phase 1: PARSE — source text to per-file ASTs.
///
/// Input:  `.urd.md` source text
/// Output: `FileAst` with span-tracked nodes
///
/// Key guarantee: every syntactically valid construct has a node.
/// Errors produce `ErrorNode` markers. The parser never aborts.
///
/// Diagnostic code range: URD100–URD199

mod frontmatter;
mod content;

use crate::ast::*;
use crate::diagnostics::DiagnosticCollector;
use crate::graph::MAX_FILE_SIZE;
use crate::span::{FilePath, Span};

/// Parse a single `.urd.md` source file into a `FileAst`.
///
/// Returns `None` only for:
/// - File exceeds 1 MB size limit (URD103)
/// - Unclosed frontmatter block (URD101)
///
/// All other errors produce `ErrorNode` markers in the AST.
pub fn parse(path: &FilePath, source: &str, diagnostics: &mut DiagnosticCollector) -> Option<FileAst> {
    // URD103: File size check (before any parsing — first thing after receiving source)
    if source.len() > MAX_FILE_SIZE {
        diagnostics.error(
            "URD103",
            format!("File exceeds 1 MB size limit: {} is {} bytes.", path, source.len()),
            Span::new(path.clone(), 1, 1, 1, 1),
        );
        return None;
    }

    // Strip UTF-8 BOM if present (after size check, before parsing)
    let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);

    let mut parser = Parser::new(path, source, diagnostics);
    parser.parse_file()
}

/// Line-oriented parser state.
pub(crate) struct Parser<'a> {
    pub file_path: String,
    pub source: &'a str,
    pub lines: Vec<LineInfo<'a>>,
    pub current_line: usize,
    pub diagnostics: &'a mut DiagnosticCollector,
}

/// Pre-computed information about a source line.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LineInfo<'a> {
    /// The raw line text (without trailing newline).
    pub text: &'a str,
    /// Byte offset of this line's start in the source.
    pub byte_offset: usize,
    /// 1-indexed line number.
    pub line_number: u32,
}

impl<'a> Parser<'a> {
    fn new(file_path: &str, source: &'a str, diagnostics: &'a mut DiagnosticCollector) -> Self {
        let lines = Self::split_lines(source);
        Parser {
            file_path: file_path.to_string(),
            source,
            lines,
            current_line: 0,
            diagnostics,
        }
    }

    /// Split source into lines, tracking byte offsets.
    fn split_lines(source: &str) -> Vec<LineInfo<'_>> {
        let mut lines = Vec::new();
        let mut offset = 0;
        let mut line_num: u32 = 1;

        for line in source.split('\n') {
            // Strip trailing \r for \r\n line endings
            let text = line.strip_suffix('\r').unwrap_or(line);
            lines.push(LineInfo {
                text,
                byte_offset: offset,
                line_number: line_num,
            });
            offset += line.len() + 1; // +1 for the \n
            line_num += 1;
        }

        lines
    }

    /// Create a span for a full line (from column 1).
    pub(crate) fn line_span(&self, line_idx: usize) -> Span {
        let info = &self.lines[line_idx];
        Span::new(
            self.file_path.clone(),
            info.line_number,
            1,
            info.line_number,
            (info.text.len() as u32) + 1, // exclusive end
        )
    }

    /// Create a span for a content line, starting after structural indent spaces.
    /// Only skips SPACE characters (structural indent). Tabs are error characters
    /// and are included in the span — per the brief: "span reflects original bytes."
    pub(crate) fn content_line_span(&self, line_idx: usize) -> Span {
        let info = &self.lines[line_idx];
        let leading_spaces = info.text.bytes().take_while(|&b| b == b' ').count();
        let start_col = (leading_spaces as u32) + 1;
        Span::new(
            self.file_path.clone(),
            info.line_number,
            start_col,
            info.line_number,
            (info.text.len() as u32) + 1, // exclusive end
        )
    }

    /// Create a span covering a range of columns on a single line.
    pub(crate) fn span_on_line(&self, line_idx: usize, start_col: u32, end_col: u32) -> Span {
        let info = &self.lines[line_idx];
        Span::new(
            self.file_path.clone(),
            info.line_number,
            start_col,
            info.line_number,
            end_col,
        )
    }

    /// Create a span covering multiple lines (from column 1).
    pub(crate) fn span_lines(&self, start_line: usize, end_line: usize) -> Span {
        let start = &self.lines[start_line];
        let end = &self.lines[end_line];
        Span::new(
            self.file_path.clone(),
            start.line_number,
            1,
            end.line_number,
            (end.text.len() as u32) + 1,
        )
    }

    /// Create a span covering multiple lines, starting after structural indent spaces
    /// on the start line. Only skips SPACE characters (not tabs).
    pub(crate) fn content_span_lines(&self, start_line: usize, end_line: usize) -> Span {
        let start = &self.lines[start_line];
        let end = &self.lines[end_line];
        let leading_spaces = start.text.bytes().take_while(|&b| b == b' ').count();
        let start_col = (leading_spaces as u32) + 1;
        Span::new(
            self.file_path.clone(),
            start.line_number,
            start_col,
            end.line_number,
            (end.text.len() as u32) + 1,
        )
    }

    /// Check if we've consumed all lines.
    pub(crate) fn at_end(&self) -> bool {
        self.current_line >= self.lines.len()
    }

    /// Peek at the current line text without advancing.
    pub(crate) fn peek_line(&self) -> Option<&'a str> {
        self.lines.get(self.current_line).map(|l| l.text)
    }

    /// Advance to the next line and return the current line's text.
    pub(crate) fn advance_line(&mut self) -> Option<&'a str> {
        if self.at_end() {
            return None;
        }
        let text = self.lines[self.current_line].text;
        self.current_line += 1;
        Some(text)
    }

    /// Scan tabs on a line, emitting URD102 for each tab found.
    /// Returns the line text with tabs replaced appropriately.
    pub(crate) fn check_tabs(&mut self, line_idx: usize) -> String {
        let info = self.lines[line_idx];
        let text = info.text;

        if !text.contains('\t') {
            return text.to_string();
        }

        let mut result = String::with_capacity(text.len());
        let mut at_indent = true;

        for (byte_pos, ch) in text.char_indices() {
            if ch == '\t' {
                let col = (byte_pos as u32) + 1;
                self.diagnostics.error(
                    "URD102",
                    format!(
                        "Tab character found at line {}, column {}. Use exactly two spaces per indent level.",
                        info.line_number, col
                    ),
                    Span::new(
                        self.file_path.clone(),
                        info.line_number,
                        col,
                        info.line_number,
                        col + 1,
                    ),
                );
                if at_indent {
                    result.push_str("  "); // two spaces at indent position
                } else {
                    result.push(' '); // one space elsewhere
                }
            } else {
                if ch != ' ' {
                    at_indent = false;
                }
                result.push(ch);
            }
        }

        result
    }

    /// Count indent level (number of 2-space indents) from the start of a string.
    /// Returns (indent_level, rest_of_line).
    pub(crate) fn measure_indent(line: &str) -> (usize, &str) {
        let mut spaces = 0;
        for ch in line.chars() {
            if ch == ' ' {
                spaces += 1;
            } else {
                break;
            }
        }
        let indent_level = spaces / 2;
        let rest = &line[spaces..];
        (indent_level, rest)
    }

    /// Strip inline comments from text content.
    /// Returns the text with any trailing ` // comment` removed and trimmed.
    pub(crate) fn strip_inline_comment(text: &str) -> &str {
        // Find " //" pattern that indicates an inline comment
        if let Some(pos) = text.find(" //") {
            text[..pos].trim_end()
        } else {
            text
        }
    }

    /// Main entry point: parse the file into a FileAst.
    fn parse_file(&mut self) -> Option<FileAst> {
        let file_span_end = if self.lines.is_empty() {
            Span::new(self.file_path.clone(), 1, 1, 1, 1)
        } else {
            let last = self.lines.len() - 1;
            self.span_lines(0, last)
        };

        // Detect frontmatter
        let first_line = self.peek_line().unwrap_or("");

        let has_frontmatter = first_line.trim_end() == "---";

        if has_frontmatter {
            // Find closing ---
            let open_line = self.current_line;
            self.current_line += 1; // skip opening ---

            let mut close_line = None;
            for i in self.current_line..self.lines.len() {
                if self.lines[i].text.trim_end() == "---" {
                    close_line = Some(i);
                    break;
                }
            }

            let close_line = match close_line {
                Some(cl) => cl,
                None => {
                    // URD101: Unclosed frontmatter
                    self.diagnostics.error(
                        "URD101",
                        "Unclosed frontmatter block. Expected closing '---'.",
                        self.line_span(open_line),
                    );
                    return None;
                }
            };

            // Parse frontmatter between delimiters
            let fm = frontmatter::parse_frontmatter(self, open_line + 1, close_line);

            // Move past closing ---
            self.current_line = close_line + 1;

            // Parse content
            let content = content::parse_content(self, 0);

            Some(FileAst {
                path: self.file_path.clone(),
                frontmatter: Some(fm),
                content,
                span: file_span_end,
            })
        } else {
            // No frontmatter — entire file is content
            let content = content::parse_content(self, 0);

            Some(FileAst {
                path: self.file_path.clone(),
                frontmatter: None,
                content,
                span: file_span_end,
            })
        }
    }
}
