//! Statement-level edits for raw Workshop source.
//!
//! [`Program::edit_statements`] is the write direction of the
//! `workshop-rs/rule-content-v1` statement contract
//! ([`crate::rules::statement`]): given a parsed program, it produces the
//! checked [`crate::source::SourceEdit`] that inserts statement nodes before
//! a statement index, replaces the statement at an index, or deletes it,
//! inside one rule's `conditions` or `actions` section. Inserted and
//! replacement statements are emitted canonically in `locale` and indented
//! like the surrounding source. Bytes outside the returned edit's range are
//! unchanged, and the edit itself re-verifies them when applied.
//!
//! Addresses are positional: [`StatementAddress`] names a rule, a statement
//! list, and an index into the public [`crate::Rule`] model — the same
//! order `rule-content-v1` exports. The call is fail-closed: an
//! out-of-range or stale address (the public model no longer matching the
//! retained source at that position), a node
//! [`statement_from_content`] refuses, an emission failure, or a rule with
//! no retained source produces an error and no edit.
//!
//! Layout and comment rules, in terms of a *statement chunk* (one statement
//! plus the standalone `"…"` annotations directly attached to it):
//!
//! - `replace` rewrites only the statement's own tokens; a leading `"…"`
//!   annotation in the same chunk stays.
//! - `delete` removes the chunk: the statement and its leading `"…"`
//!   annotations. A chunk alone on its line takes the whole line, with a
//!   `//` tail comment; a chunk that shares its line takes only its own
//!   bytes and one adjacent whitespace run.
//! - `insert` writes no comments. Inserted rows take the anchor line's
//!   indentation; `//` line comments are unowned positional trivia and are
//!   never counted as part of a statement.
//! - Inserting into a rule with no `conditions` section creates the section
//!   immediately before `actions` (or the rule's closing brace when there
//!   is no `actions` section).
//! - Deleting the last statement leaves the empty section in place; it
//!   reparses with an empty list.

use std::fmt;

use serde_json::Value as JsonValue;

use crate::catalog::{Catalog, Locale};
use crate::core::error::WorkshopError;
use crate::core::source::{Position, Span};
use crate::frontend::keywords::structural_keyword_prefix;
use crate::frontend::lexer::{Token, tokenize};
use crate::program::{Action, Program};

mod emit;
mod layout;

use crate::rules::statement::{Statement, StatementContentError, statement_from_content};
use emit::{emit_section_keyword, emit_statement_rows};
use layout::{
    RuleLayout, StatementChunk, StatementSection, byte_is_whitespace, indent_unit, is_whitespace,
    line_prefix, newline_of,
};

/// The rule statement list a statement edit addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StatementList {
    /// The rule's `conditions` section (`rule.conditions`).
    Conditions,
    /// The rule's `actions` section (`rule.actions`).
    Actions,
}

impl StatementList {
    fn section_id(self) -> &'static str {
        match self {
            Self::Conditions => "conditions",
            Self::Actions => "actions",
        }
    }

    fn list_len(self, rule: &crate::Rule) -> usize {
        match self {
            Self::Conditions => rule.conditions.len(),
            Self::Actions => rule.actions.len(),
        }
    }
}

/// The `(rule, list, index)` address of a statement edit, by position in the
/// public [`crate::Rule`] model — the same order `rule-content-v1` exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct StatementAddress {
    /// The rule index in `program.rules`.
    pub rule: usize,
    /// The statement list inside the rule.
    pub list: StatementList,
    /// The statement index inside the list. For an insert this is the index
    /// the new statements are placed before; `index == list.len()` appends.
    pub index: usize,
}

impl StatementAddress {
    /// An address by rule index, statement list, and statement index.
    pub fn new(rule: usize, list: StatementList, index: usize) -> Self {
        Self { rule, list, index }
    }
}

/// One edit against a rule statement list, addressed by
/// [`StatementAddress`]. The payload nodes are `rule-content-v1` statement
/// documents — `{"value": …}` for conditions, `{"call": id, "args": […]}`
/// for actions — converted through
/// [`statement_from_content`].
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum StatementEdit<'a> {
    /// Insert a non-empty list of statements before `index`
    /// (`index == list.len()` appends at the end of the section).
    Insert(&'a [JsonValue]),
    /// Replace exactly the statement at `index`, keeping its leading
    /// comments.
    Replace(&'a JsonValue),
    /// Delete the statement at `index` together with its comments.
    Delete,
}

/// Why [`Program::edit_statements`] refused an edit. Every refusal produces
/// no [`crate::source::SourceEdit`].
#[derive(Debug)]
#[non_exhaustive]
pub enum StatementEditError {
    /// A statement node `statement_from_content` could not convert.
    Content(StatementContentError),
    /// `address.rule` indexes no rule of the program.
    UnknownRule {
        /// The out-of-range rule index.
        rule: usize,
    },
    /// `address.index` addresses no statement: past the end of the list, or
    /// into the synthesized tail a rule-final unclosed `if` group carries in
    /// the public model but not in the source.
    IndexOutOfRange {
        /// The rule index.
        rule: usize,
        /// The addressed list.
        list: StatementList,
        /// The out-of-range statement index.
        index: usize,
    },
    /// A condition node was addressed at `actions`, or an action node at
    /// `conditions`.
    StatementKind {
        /// The rule index.
        rule: usize,
        /// The addressed list.
        list: StatementList,
    },
    /// `insert` was given an empty statement list.
    EmptyInsert {
        /// The rule index.
        rule: usize,
        /// The addressed list.
        list: StatementList,
    },
    /// The program carries no retained source for the addressed rule, or
    /// the retained text can no longer be read back.
    SourceUnavailable {
        /// The rule index.
        rule: usize,
    },
    /// The retained source does not match the public model at the addressed
    /// position: the statement count differs, the addressed index is a
    /// source-synthesized entry, or the statement recorded there no longer
    /// has the model's content.
    StaleAddress {
        /// The rule index.
        rule: usize,
        /// The addressed list.
        list: StatementList,
    },
    /// Statement emission failed (for example a spelling missing in the
    /// requested locale, or a statement the Workshop grammar cannot carry).
    Emission(WorkshopError),
    /// The computed edit failed `Program::edit_source`'s checked range.
    Edit(crate::source::SourceEditError),
}

impl fmt::Display for StatementEditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Content(error) => write!(f, "{error}"),
            Self::UnknownRule { rule } => write!(f, "no rule at index {rule}"),
            Self::IndexOutOfRange { rule, list, index } => {
                write!(f, "no {list:?} statement at index {index} in rule {rule}")
            }
            Self::StatementKind { rule, list } => write!(
                f,
                "statement kind does not match the {list:?} list of rule {rule}"
            ),
            Self::EmptyInsert { rule, list } => {
                write!(f, "insert into {list:?} of rule {rule} needs a statement")
            }
            Self::SourceUnavailable { rule } => {
                write!(f, "rule {rule} has no editable source")
            }
            Self::StaleAddress { rule, list } => write!(
                f,
                "the {list:?} of rule {rule} no longer matches its retained source"
            ),
            Self::Emission(error) => write!(f, "statement emission failed: {error}"),
            Self::Edit(error) => write!(f, "source edit failed: {error}"),
        }
    }
}

impl std::error::Error for StatementEditError {}

impl From<StatementContentError> for StatementEditError {
    fn from(error: StatementContentError) -> Self {
        Self::Content(error)
    }
}

impl From<WorkshopError> for StatementEditError {
    fn from(error: WorkshopError) -> Self {
        Self::Emission(error)
    }
}

impl From<crate::source::SourceEditError> for StatementEditError {
    fn from(error: crate::source::SourceEditError) -> Self {
        Self::Edit(error)
    }
}

type Result<T> = std::result::Result<T, StatementEditError>;

/// The structural keyword ids an actions chunk can open with. `else`/`else
/// if`/`end` have no recorded span, so this is how their chunks verify the
/// model entry kind at the addressed position.
const ACTION_KEYWORD_IDS: &[&str] = &[
    "disabled",
    "if",
    "elseIf",
    "else",
    "while",
    "forGlobalVariable",
    "forPlayerVariable",
    "end",
];

impl Program {
    /// Produce the checked [`crate::source::SourceEdit`] that inserts,
    /// replaces, or deletes one statement position of `rule`'s
    /// `conditions` or `actions` list. See the module documentation for the
    /// address model and the layout/comment rules.
    ///
    /// `locale` is the session's resolved Workshop locale: inserted and
    /// replacement statements are emitted with its spellings, and missing
    /// spellings fail explicitly. The returned edit is checked again on
    /// apply — [`crate::source::SourceDocument::apply`] refuses it if the
    /// source bytes at its range changed — and consumers reparse the edited
    /// text for updated spans.
    ///
    /// ```
    /// use serde_json::json;
    /// use workshop_rs::catalog::{Catalog, Locale};
    /// use workshop_rs::parser;
    /// use workshop_rs::rules::statement_edit::{
    ///     StatementAddress, StatementEdit, StatementList,
    /// };
    ///
    /// let catalog = Catalog::builtin().unwrap();
    /// let source = "rule (\"x\") {\n    event { Ongoing - Global; }\n    actions { Wait(1, Ignore Condition); }\n}\n";
    /// let program = parser::parse(source, &catalog, &Locale::new("en-US")).unwrap();
    /// let edit = program
    ///     .edit_statements(
    ///         &catalog,
    ///         &Locale::new("en-US"),
    ///         StatementAddress::new(0, StatementList::Actions, 0),
    ///         StatementEdit::Replace(&json!({"call": "wait", "args": [2.0]})),
    ///     )
    ///     .unwrap();
    /// let updated = edit.apply(source).unwrap();
    /// assert!(updated.contains("Wait(2);"));
    /// ```
    pub fn edit_statements(
        &self,
        catalog: &Catalog,
        locale: &Locale,
        address: StatementAddress,
        edit: StatementEdit<'_>,
    ) -> Result<crate::source::SourceEdit> {
        let rule = self
            .rules
            .get(address.rule)
            .ok_or(StatementEditError::UnknownRule { rule: address.rule })?;

        let statements = convert_statements(catalog, address, &edit)?;
        let list_len = address.list.list_len(rule);
        match edit {
            StatementEdit::Insert(_) => {
                if address.index > list_len {
                    return Err(StatementEditError::IndexOutOfRange {
                        rule: address.rule,
                        list: address.list,
                        index: address.index,
                    });
                }
            }
            StatementEdit::Replace(_) | StatementEdit::Delete => {
                if address.index >= list_len {
                    return Err(StatementEditError::IndexOutOfRange {
                        rule: address.rule,
                        list: address.list,
                        index: address.index,
                    });
                }
            }
        }

        let rows = emit_statement_rows(&statements, self, catalog, locale)?;

        let rule_span = self
            .rule_span(address.rule)
            .ok_or(StatementEditError::SourceUnavailable { rule: address.rule })?;
        let document = self
            .source(rule_span.file)
            .ok_or(StatementEditError::SourceUnavailable { rule: address.rule })?;
        let text = document.text();
        let tokens = tokenize(text)
            .map_err(|_| StatementEditError::SourceUnavailable { rule: address.rule })?;
        let layout = RuleLayout::scan(&tokens, rule_span, catalog, locale)
            .ok_or(StatementEditError::SourceUnavailable { rule: address.rule })?;

        let section = layout
            .sections
            .iter()
            .find(|section| section.id == address.list.section_id());
        let Some(section) = section else {
            let unit = indent_unit(text, &tokens, &layout, rule_span);
            return self.insert_created_section(
                catalog, locale, address, list_len, &rows, &unit, &tokens, &layout, text, rule_span,
            );
        };

        let synthesized_tail = match address.list {
            StatementList::Conditions => section.chunks.len() != list_len,
            StatementList::Actions => {
                section.chunks.len() > list_len
                    || !rule.actions[section.chunks.len()..]
                        .iter()
                        .all(|action| matches!(action, Action::End))
            }
        };
        if synthesized_tail {
            return Err(StatementEditError::StaleAddress {
                rule: address.rule,
                list: address.list,
            });
        }

        match edit {
            StatementEdit::Insert(_) => {
                // Past the last source chunk is the end of the section. With a
                // synthesized trailing `End` that covers `index == len` and the
                // `End`'s own index: both append after the last chunk.
                if let Some(chunk) = section.chunks.get(address.index) {
                    self.verify_statement(
                        address,
                        *chunk,
                        &tokens,
                        catalog,
                        locale,
                        rule_span.file,
                    )?;
                    self.insert_before_chunk(address, &rows, *chunk, &tokens, text, rule_span)
                } else {
                    let unit = indent_unit(text, &tokens, &layout, rule_span);
                    self.insert_at_close(address, &rows, &unit, section, &tokens, text, rule_span)
                }
            }
            StatementEdit::Replace(_) => {
                let chunk =
                    section
                        .chunks
                        .get(address.index)
                        .ok_or(StatementEditError::StaleAddress {
                            rule: address.rule,
                            list: address.list,
                        })?;
                self.verify_statement(address, *chunk, &tokens, catalog, locale, rule_span.file)?;
                let row = rows.first().expect("replace emits one row");
                self.replace_chunk(address, row, *chunk, &tokens, rule_span)
            }
            StatementEdit::Delete => {
                let chunk =
                    section
                        .chunks
                        .get(address.index)
                        .ok_or(StatementEditError::StaleAddress {
                            rule: address.rule,
                            list: address.list,
                        })?;
                self.verify_statement(address, *chunk, &tokens, catalog, locale, rule_span.file)?;
                self.delete_chunk(address, *chunk, &tokens, text, rule_span)
            }
        }
    }

    /// The retained source ties the model entry at `address.index` to
    /// `chunk`: span-bearing entries resolve through the recorded spans,
    /// spanless `Else`/`End` markers through the chunk's leading keyword.
    fn verify_statement(
        &self,
        address: StatementAddress,
        chunk: StatementChunk,
        tokens: &[Token],
        catalog: &Catalog,
        locale: &Locale,
        file: crate::source::FileId,
    ) -> Result<()> {
        let chunk_start = tokens[chunk.first_content].start;
        let chunk_end = tokens[chunk.semi].end;
        let within = |start: Position| {
            (start.line, start.col) >= (chunk_start.line, chunk_start.col)
                && (start.line, start.col) <= (chunk_end.line, chunk_end.col)
        };
        let verified = match address.list {
            StatementList::Conditions => self
                .condition_span(address.rule, address.index)
                .is_some_and(|span| span.file == file && within(span.start)),
            StatementList::Actions => match &self.rules[address.rule].actions[address.index] {
                Action::Else | Action::End => {
                    let expected = if matches!(
                        &self.rules[address.rule].actions[address.index],
                        Action::Else
                    ) {
                        "else"
                    } else {
                        "end"
                    };
                    structural_keyword_prefix(
                        tokens,
                        chunk.first_content,
                        catalog,
                        locale,
                        ACTION_KEYWORD_IDS,
                    )
                    .is_some_and(|(id, _)| id == expected)
                }
                Action::ElseIf { .. } => self
                    .action_argument_span(address.rule, address.index, 0)
                    .is_some_and(|span| span.file == file && within(span.start)),
                _ => self
                    .action_span(address.rule, address.index)
                    .is_some_and(|span| span.file == file && within(span.start)),
            },
        };
        if verified {
            Ok(())
        } else {
            Err(StatementEditError::StaleAddress {
                rule: address.rule,
                list: address.list,
            })
        }
    }

    /// Insert `rows` on their own lines before `chunk` — or inline before
    /// it when the chunk does not start its line.
    fn insert_before_chunk(
        &self,
        address: StatementAddress,
        rows: &[String],
        chunk: StatementChunk,
        tokens: &[Token],
        text: &str,
        rule_span: Span,
    ) -> Result<crate::source::SourceEdit> {
        let anchor = tokens[chunk.first].start;
        let newline = newline_of(text, anchor);
        let line_start = Position::new(anchor.line, 1);
        let indent = line_prefix(text, anchor);
        let (position, replacement) = if is_whitespace(indent) {
            let mut replacement = String::new();
            for row in rows {
                replacement.push_str(indent);
                replacement.push_str(row);
                replacement.push_str(newline);
            }
            (line_start, replacement)
        } else {
            let left = if byte_is_whitespace(text, anchor) {
                ""
            } else {
                " "
            };
            (anchor, format!("{left}{} ", rows.join(" ")))
        };
        self.checked_edit(rule_span, position, position, replacement)
            .map_err(|error| wrap_edit_error(address, error))
    }

    /// Append `rows` at the end of an existing (possibly empty) section,
    /// before its closing `}`.
    #[allow(clippy::too_many_arguments)]
    fn insert_at_close(
        &self,
        address: StatementAddress,
        rows: &[String],
        unit: &str,
        section: &StatementSection,
        tokens: &[Token],
        text: &str,
        rule_span: Span,
    ) -> Result<crate::source::SourceEdit> {
        // Annotations after the last statement have no statement to attach
        // to: the new statement goes before them so it does not take them.
        let close = tokens[section.dangling.unwrap_or(section.close)].start;
        let newline = newline_of(text, close);
        let close_indent = line_prefix(text, close);
        let (position, replacement) = if is_whitespace(close_indent) {
            let indent = section
                .chunks
                .last()
                .map(|chunk| line_prefix(text, tokens[chunk.first].start))
                .filter(|prefix| is_whitespace(prefix))
                .map(str::to_string)
                .unwrap_or_else(|| format!("{close_indent}{unit}"));
            let mut replacement = String::new();
            for row in rows {
                replacement.push_str(&indent);
                replacement.push_str(row);
                replacement.push_str(newline);
            }
            (Position::new(close.line, 1), replacement)
        } else {
            let left = if byte_is_whitespace(text, close) {
                ""
            } else {
                " "
            };
            (close, format!("{left}{} ", rows.join(" ")))
        };
        self.checked_edit(rule_span, position, position, replacement)
            .map_err(|error| wrap_edit_error(address, error))
    }

    /// Insert into a rule whose section does not exist: create the section
    /// immediately before `actions` when it exists, else before the rule's
    /// closing `}`. Only reachable for an empty-list insert — any other
    /// address was already refused as out of range or stale.
    #[allow(clippy::too_many_arguments)]
    fn insert_created_section(
        &self,
        catalog: &Catalog,
        locale: &Locale,
        address: StatementAddress,
        list_len: usize,
        rows: &[String],
        unit: &str,
        tokens: &[Token],
        layout: &RuleLayout,
        text: &str,
        rule_span: Span,
    ) -> Result<crate::source::SourceEdit> {
        // A section can only be absent together with its list: model
        // statements with no section to hold them mean the model and the
        // retained source disagree.
        if list_len != 0 || address.index != 0 || rows.is_empty() {
            return Err(if list_len == 0 {
                StatementEditError::IndexOutOfRange {
                    rule: address.rule,
                    list: address.list,
                    index: address.index,
                }
            } else {
                StatementEditError::StaleAddress {
                    rule: address.rule,
                    list: address.list,
                }
            });
        }
        let keyword = emit_section_keyword(address.list, catalog, locale)?;
        let anchor = layout
            .sections
            .iter()
            .find(|section| section.id == "actions")
            .map(|section| section.name_start)
            .unwrap_or_else(|| tokens[layout.close].start);
        let newline = newline_of(text, anchor);
        // The section indent follows the sibling sections (or, for a rule
        // with no sections at all, one level inside the closing brace).
        let section_indent = layout
            .sections
            .last()
            .map(|section| line_prefix(text, section.name_start))
            .filter(|prefix| is_whitespace(prefix))
            .map(str::to_string);
        let anchor_indent = line_prefix(text, anchor);
        let (position, replacement) = if is_whitespace(anchor_indent) {
            let indent = section_indent.unwrap_or_else(|| format!("{anchor_indent}{unit}"));
            let statement_indent = format!("{indent}{unit}");
            let mut replacement = format!("{indent}{keyword} {{{newline}");
            for row in rows {
                replacement.push_str(&statement_indent);
                replacement.push_str(row);
                replacement.push_str(newline);
            }
            replacement.push_str(&indent);
            replacement.push('}');
            replacement.push_str(newline);
            (Position::new(anchor.line, 1), replacement)
        } else {
            let left = if byte_is_whitespace(text, anchor) {
                ""
            } else {
                " "
            };
            (anchor, format!("{left}{keyword} {{ {} }} ", rows.join(" ")))
        };
        self.checked_edit(rule_span, position, position, replacement)
            .map_err(|error| wrap_edit_error(address, error))
    }

    /// Replace one statement's own tokens: `chunk.first_content`..`chunk.semi`
    /// — leading `"…"` annotations stay where they are.
    fn replace_chunk(
        &self,
        address: StatementAddress,
        row: &str,
        chunk: StatementChunk,
        tokens: &[Token],
        rule_span: Span,
    ) -> Result<crate::source::SourceEdit> {
        let start = tokens[chunk.first_content].start;
        let end = tokens[chunk.semi].end;
        self.checked_edit(rule_span, start, end, row)
            .map_err(|error| wrap_edit_error(address, error))
    }

    /// Delete one chunk: leading annotations and the statement. A chunk
    /// alone on its line takes the whole line, including a `//` tail
    /// comment. A chunk that shares its line takes only its own bytes and
    /// one adjacent whitespace run, so the other tokens, the indentation,
    /// and a `//` comment after them stay.
    fn delete_chunk(
        &self,
        address: StatementAddress,
        chunk: StatementChunk,
        tokens: &[Token],
        text: &str,
        rule_span: Span,
    ) -> Result<crate::source::SourceEdit> {
        let first = tokens[chunk.first].start;
        let semi_end = tokens[chunk.semi].end;
        let starts_line = is_whitespace(line_prefix(text, first));
        // `//` comments are not tokens, so a later token on the same line
        // means real content follows the statement.
        let next = tokens.get(chunk.semi + 1).map(|token| token.start);
        let content_follows = next.is_some_and(|next| next.line == semi_end.line);
        let (start, end) = match (starts_line, content_follows) {
            (true, false) => (
                Position::new(first.line, 1),
                Position::new(semi_end.line + 1, 1),
            ),
            (_, true) => (first, next.expect("content follows")),
            (false, false) => {
                let blanks = line_prefix(text, first)
                    .chars()
                    .rev()
                    .take_while(|character| matches!(character, ' ' | '\t'))
                    .count();
                (
                    Position::new(first.line, first.col - blanks as u32),
                    semi_end,
                )
            }
        };
        self.checked_edit(rule_span, start, end, "")
            .map_err(|error| wrap_edit_error(address, error))
    }

    fn checked_edit(
        &self,
        rule_span: Span,
        start: Position,
        end: Position,
        replacement: impl Into<String>,
    ) -> std::result::Result<crate::source::SourceEdit, crate::source::SourceEditError> {
        self.edit_source(Span::new(rule_span.file, start, end), replacement)
    }
}

fn wrap_edit_error(
    address: StatementAddress,
    error: crate::source::SourceEditError,
) -> StatementEditError {
    match error {
        crate::source::SourceEditError::InvalidRange => {
            StatementEditError::SourceUnavailable { rule: address.rule }
        }
        other => StatementEditError::Edit(other),
    }
}

fn convert_statements(
    catalog: &Catalog,
    address: StatementAddress,
    edit: &StatementEdit<'_>,
) -> Result<Vec<Statement>> {
    let nodes: &[JsonValue] = match edit {
        StatementEdit::Insert(nodes) => {
            if nodes.is_empty() {
                return Err(StatementEditError::EmptyInsert {
                    rule: address.rule,
                    list: address.list,
                });
            }
            nodes
        }
        StatementEdit::Replace(node) => std::slice::from_ref(node),
        StatementEdit::Delete => &[],
    };
    let mut statements = Vec::with_capacity(nodes.len());
    for node in nodes {
        let statement = statement_from_content(catalog, node)?;
        let fits = matches!(
            (address.list, &statement),
            (StatementList::Conditions, Statement::Condition(_))
                | (StatementList::Actions, Statement::Action(_))
        );
        if !fits {
            return Err(StatementEditError::StatementKind {
                rule: address.rule,
                list: address.list,
            });
        }
        statements.push(statement);
    }
    Ok(statements)
}
