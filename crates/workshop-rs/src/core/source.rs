//! Source model: files, positions, and spans.
//!
//! Conventions: positions are 1-based, and a span is a half-open interval
//! (`end` is exclusive). Spans carry a typed [`FileId`] instead of a raw file
//! index.

use std::{ops::Range, slice::Iter};

use super::ids::Id;

/// A typed ID referencing a [`SourceFile`] in the program's file arena.
pub type FileId = Id<SourceFile>;

/// One source file in the program's file registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// The file name as the frontend reported it (for diagnostics).
    pub path: String,
    file: Option<FileId>,
    source: Option<SourceDocument>,
}

impl SourceFile {
    /// Create a file entry with the given path.
    pub fn new(path: impl Into<String>) -> Self {
        SourceFile {
            path: path.into(),
            file: None,
            source: None,
        }
    }

    /// Create a file entry that retains its authored source text and comments.
    pub fn with_source(path: impl Into<String>, source: impl Into<String>) -> Self {
        SourceFile {
            path: path.into(),
            file: None,
            source: Some(SourceDocument::new(source)),
        }
    }

    /// Attach authored source to an existing file entry.
    pub fn set_source(&mut self, source: impl Into<String>) {
        let mut document = SourceDocument::new(source);
        document.file = self.file;
        self.source = Some(document);
    }

    pub(crate) fn bind_file(&mut self, file: FileId) {
        self.file = Some(file);
        if let Some(source) = &mut self.source {
            source.file = Some(file);
        }
    }

    /// The retained authored source, when this file was created source-aware.
    pub fn source(&self) -> Option<&SourceDocument> {
        self.source.as_ref()
    }
}

/// A source document retained independently from canonical semantic nodes.
///
/// Whitespace and other non-comment trivia remain in [`Self::text`]. Comments
/// are indexed as a convenience for stable span-based attachment; callers
/// should use [`SourceEdit`] for local changes and reparse the edited text to
/// obtain updated semantic spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDocument {
    file: Option<FileId>,
    text: String,
    comments: Vec<SourceComment>,
}

impl SourceDocument {
    /// Retain source text and index supported line comments (`// ...`).
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let comments = find_line_comments(&text);
        Self {
            file: None,
            text,
            comments,
        }
    }

    /// The exact authored source text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// All indexed comments in authored source order.
    pub fn comments(&self) -> Iter<'_, SourceComment> {
        self.comments.iter()
    }

    /// Comments whose complete source range is inside a semantic span.
    ///
    /// This is the attachment rule for the supported slice. Comments outside
    /// a node span remain document-level preserved source and are never
    /// guessed into a neighboring node.
    pub fn comments_for(&self, span: Span) -> impl Iterator<Item = &SourceComment> {
        let range = self.byte_range(span);
        self.comments.iter().filter(move |comment| {
            range.as_ref().is_some_and(|range| {
                comment.range.start >= range.start && comment.range.end <= range.end
            })
        })
    }

    /// Convert a line/column span into a UTF-8 byte range in this document.
    pub fn byte_range(&self, span: Span) -> Option<Range<usize>> {
        if self.file != Some(span.file) {
            return None;
        }
        let start = byte_offset(&self.text, span.start)?;
        let end = byte_offset(&self.text, span.end)?;
        (start <= end).then_some(start..end)
    }

    /// Create a checked replacement for a UTF-8 byte range.
    pub fn edit(
        &self,
        range: Range<usize>,
        replacement: impl Into<String>,
    ) -> Result<SourceEdit, SourceEditError> {
        if range.start > range.end
            || !self.text.is_char_boundary(range.start)
            || !self.text.is_char_boundary(range.end)
            || range.end > self.text.len()
        {
            return Err(SourceEditError::InvalidRange);
        }
        Ok(SourceEdit {
            expected: self.text[range.clone()].to_string(),
            range,
            replacement: replacement.into(),
        })
    }

    /// Create a checked replacement for a semantic span.
    pub fn edit_span(
        &self,
        span: Span,
        replacement: impl Into<String>,
    ) -> Result<SourceEdit, SourceEditError> {
        let range = self.byte_range(span).ok_or(SourceEditError::InvalidRange)?;
        self.edit(range, replacement)
    }

    /// Apply non-overlapping edits against this exact document.
    pub fn apply(&self, edits: &[SourceEdit]) -> Result<Self, SourceEditError> {
        let mut ordered = edits.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|edit| edit.range.start);
        for pair in ordered.windows(2) {
            if pair[0].range.end > pair[1].range.start || pair[0].range.start == pair[1].range.start
            {
                return Err(SourceEditError::OverlappingEdits);
            }
        }
        let mut text = self.text.clone();
        for edit in ordered.into_iter().rev() {
            edit.apply_to(&mut text)?;
        }
        Ok(Self::new(text))
    }
}

/// A source comment retained from the authored document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceComment {
    kind: CommentKind,
    range: Range<usize>,
}

impl SourceComment {
    pub fn kind(&self) -> CommentKind {
        self.kind
    }

    /// The UTF-8 byte range including the `//` marker and excluding its line
    /// ending.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// The exact comment text from the containing document.
    pub fn text<'a>(&self, document: &'a SourceDocument) -> &'a str {
        &document.text[self.range.clone()]
    }
}

/// Comment kinds currently supported by the raw Workshop source contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Line,
}

/// A checked, byte-oriented source replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEdit {
    range: Range<usize>,
    expected: String,
    replacement: String,
}

impl SourceEdit {
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn replacement(&self) -> &str {
        &self.replacement
    }

    /// Apply this edit only when the original bytes still match.
    pub fn apply(&self, source: &str) -> Result<String, SourceEditError> {
        let mut result = source.to_string();
        self.apply_to(&mut result)?;
        Ok(result)
    }

    fn apply_to(&self, source: &mut String) -> Result<(), SourceEditError> {
        if source.get(self.range.clone()) != Some(self.expected.as_str()) {
            return Err(SourceEditError::SourceMismatch);
        }
        source.replace_range(self.range.clone(), &self.replacement);
        Ok(())
    }
}

/// Failure while creating or applying source edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceEditError {
    InvalidRange,
    SourceMismatch,
    OverlappingEdits,
}

impl std::fmt::Display for SourceEditError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidRange => "source edit range is not a valid UTF-8 range",
            Self::SourceMismatch => "source no longer matches the edit",
            Self::OverlappingEdits => "source edits overlap",
        })
    }
}

impl std::error::Error for SourceEditError {}

fn find_line_comments(source: &str) -> Vec<SourceComment> {
    let mut comments = Vec::new();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;
    while index < source.len() {
        let character = source[index..].chars().next().unwrap();
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            index += character.len_utf8();
            continue;
        }
        if character == '"' {
            in_string = true;
            index += character.len_utf8();
        } else if character == '/' && source[index..].starts_with("//") {
            let start = index;
            index += 2;
            while index < source.len()
                && !source[index..].starts_with('\n')
                && !source[index..].starts_with('\r')
            {
                index += source[index..].chars().next().unwrap().len_utf8();
            }
            comments.push(SourceComment {
                kind: CommentKind::Line,
                range: start..index,
            });
        } else {
            index += character.len_utf8();
        }
    }
    comments
}

fn byte_offset(source: &str, position: Position) -> Option<usize> {
    if !position.is_valid() {
        return None;
    }
    let mut line = 1;
    let mut col = 1;
    for (index, character) in source.char_indices() {
        if line == position.line && col == position.col {
            return Some(index);
        }
        if character == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line == position.line && col == position.col).then_some(source.len())
}

/// A 1-based line/column position in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub col: u32,
}

impl Position {
    /// A position at line `line`, column `col` (both 1-based).
    pub const fn new(line: u32, col: u32) -> Self {
        Position { line, col }
    }

    /// Whether this position is valid (1-based).
    pub const fn is_valid(self) -> bool {
        self.line >= 1 && self.col >= 1
    }
}

/// A half-open, 1-based source interval in one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file: FileId,
    pub start: Position,
    pub end: Position,
}

impl Span {
    /// Create a span in `file` from `start` (inclusive) to `end` (exclusive).
    pub const fn new(file: FileId, start: Position, end: Position) -> Self {
        Span { file, start, end }
    }

    /// Whether the span is structurally valid: both positions are 1-based and
    /// `end` is not before `start`.
    pub const fn is_valid(self) -> bool {
        self.start.is_valid()
            && self.end.is_valid()
            && (self.end.line > self.start.line
                || (self.end.line == self.start.line && self.end.col >= self.start.col))
    }
}

#[cfg(test)]
mod tests {
    use super::super::ids::Id;
    use super::{CommentKind, Position, SourceDocument, SourceFile, Span};

    #[test]
    fn positions_are_one_based_and_validated() {
        assert!(Position::new(1, 1).is_valid());
        assert!(Position::new(10, 24).is_valid());
        assert!(!Position::new(0, 1).is_valid());
        assert!(!Position::new(1, 0).is_valid());
    }

    #[test]
    fn spans_require_end_not_before_start() {
        let file = Id::from_index(0);
        assert!(Span::new(file, Position::new(1, 1), Position::new(1, 5)).is_valid());
        assert!(Span::new(file, Position::new(1, 1), Position::new(2, 1)).is_valid());
        assert!(Span::new(file, Position::new(1, 1), Position::new(1, 1)).is_valid());
        assert!(!Span::new(file, Position::new(1, 5), Position::new(1, 1)).is_valid());
        assert!(!Span::new(file, Position::new(2, 1), Position::new(1, 1)).is_valid());
    }

    #[test]
    fn source_files_carry_paths() {
        let file = SourceFile::new("source.opy");
        assert_eq!(file.path, "source.opy");
        assert!(file.source().is_none());
    }

    #[test]
    fn source_documents_index_line_comments_but_not_string_contents() {
        let document = SourceDocument::new("// before\nWait(\"// not a comment\"); // after\n");
        let comments: Vec<_> = document.comments().collect();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].kind(), CommentKind::Line);
        assert_eq!(comments[0].text(&document), "// before");
        assert_eq!(comments[1].text(&document), "// after");
    }

    #[test]
    fn source_edits_are_checked_and_reindex_comments() {
        let document = SourceDocument::new("// keep\nvalue: 1\n");
        let edit = document.edit(15..16, "2").expect("valid edit");
        let updated = document.apply(&[edit]).expect("edit applies");
        assert_eq!(updated.text(), "// keep\nvalue: 2\n");
        assert_eq!(updated.comments().count(), 1);
    }

    #[test]
    fn source_edits_reject_stale_and_overlapping_inputs() {
        let document = SourceDocument::new("abcdef");
        let edit = document.edit(1..3, "x").unwrap();
        assert!(matches!(
            edit.apply("aXcdef"),
            Err(super::SourceEditError::SourceMismatch)
        ));
        let left = document.edit(1..3, "x").unwrap();
        let right = document.edit(2..4, "y").unwrap();
        assert!(matches!(
            document.apply(&[left, right]),
            Err(super::SourceEditError::OverlappingEdits)
        ));
    }

    #[test]
    fn source_comment_ranges_exclude_crlf_line_endings() {
        let document = SourceDocument::new("// comment\r\nnext\r\n");
        let comment = document.comments().next().unwrap();
        assert_eq!(comment.text(&document), "// comment");
    }
}
