use crate::output::emitter::*;

impl EmitContext<'_> {
    pub(crate) fn value(&mut self, id: wir::ValueId, out: &mut String) -> Result<()> {
        let Some(node) = self.program.values.get(id) else {
            return Err(WorkshopError::Malformed {
                message: format!("dangling value {id}"),
                span: None,
            });
        };
        match &node.value {
            wir::Value::Number { text, .. } => {
                // Literal spellings carry through (the oracle preserves the
                // source spelling, e.g. `0.0`; computed values carry the
                // formatted spelling, #87).
                out.push_str(text);
            }
            wir::Value::String(value) => {
                // Value-position strings wrap in `Custom String("...")` with
                // re-escaped content and long-string splitting, the pinned
                // oracle's spelling (evidence: array elements, initializers,
                // assignments, call arguments, comparisons — #87). The only
                // bare string value is the `Custom String` text argument,
                // handled in the call arm below.
                self.emit_string_value(value, out)?;
            }
            wir::Value::LocalizedString(id) => {
                out.push_str(&self.spelling(Kind::Value, "string")?);
                out.push('(');
                let spelling = self.localized_string_spelling(id)?;
                write!(out, "\"{}\"", escape_value_string(&spelling)).unwrap();
                out.push(')');
            }
            wir::Value::Bool(true) => out.push_str("True"),
            wir::Value::Bool(false) => out.push_str("False"),
            wir::Value::Null => out.push_str("Null"),
            wir::Value::Array(elements) => {
                if elements.is_empty() {
                    // The canonical empty-array constant (reference emission).
                    out.push_str(&self.spelling(Kind::Value, "emptyArray")?);
                } else {
                    out.push_str(&self.spelling(Kind::Value, "array")?);
                    out.push('(');
                    self.args(elements, out)?;
                    out.push(')');
                }
            }
            wir::Value::Vector { x, y, z } => {
                out.push_str(&self.spelling(Kind::Value, "vector")?);
                out.push('(');
                self.value(*x, out)?;
                out.push_str(", ");
                self.value(*y, out)?;
                out.push_str(", ");
                self.value(*z, out)?;
                out.push(')');
            }
            wir::Value::Enum { value_type, value } => {
                let spelling = self.enum_spelling(value_type, value)?;
                // Color, Team, and Hero values use the constructor form;
                // other domains use bare member spellings (the canonical
                // corpus form). The
                // Team/Color spelling collision (`Team 2` is both a Team and
                // a Team color) is the one ambiguity unpinned by the
                // catalog's paramDomains, so Team members qualify with the
                // constructor form and the emitted text reparses
                // deterministically (round-trip contract; pinned P4
                // evidence).
                if matches!(value_type.as_str(), "Color" | "Map" | "Team")
                    || value_type == "Hero"
                        && (spelling.contains('.')
                            || self.locale != *self.catalog.primary_locale()
                            || (self.force_hero_constructors
                                && self
                                    .program
                                    .global_variables
                                    .iter()
                                    .any(|variable| variable.name == spelling)))
                {
                    let domain = self
                        .catalog
                        .enum_domain(value_type)
                        .and_then(|entry| entry.spelling(&self.locale))
                        .unwrap_or(value_type);
                    write!(out, "{domain}({spelling})").unwrap();
                } else {
                    out.push_str(&spelling);
                }
            }
            wir::Value::GlobalVariable(variable) => {
                let name = self.global_name(*variable)?;
                write!(out, "Global.{name}").unwrap();
            }
            wir::Value::PlayerVariable { player, variable } => {
                // The oracle's spelling parenthesizes the receiver:
                // `Set Global Variable(g, (Event Player).p)` (#87).
                out.push('(');
                self.value(*player, out)?;
                out.push(')');
                let name = self.player_name(*variable)?;
                write!(out, ".{name}").unwrap();
            }
            wir::Value::Subroutine(subroutine) => {
                let name = self
                    .program
                    .subroutines
                    .get(*subroutine)
                    .map(|value| value.name.clone())
                    .ok_or_else(|| WorkshopError::Malformed {
                        message: format!("dangling subroutine value {subroutine}"),
                        span: None,
                    })?;
                out.push_str(&name);
            }
            wir::Value::EventPlayer => out.push_str(&self.spelling(Kind::Value, "eventPlayer")?),
            wir::Value::Call { name, args } => {
                if name == "memberAccess" {
                    if args.len() < 2 || args.len() > 3 {
                        return Err(WorkshopError::Malformed {
                            message: "memberAccess expects two or three arguments".to_string(),
                            span: node.span,
                        });
                    }
                    let Some(wir::ValueNode {
                        value: wir::Value::String(member),
                        ..
                    }) = self.program.values.get(args[1])
                    else {
                        return Err(WorkshopError::Malformed {
                            message: "memberAccess member must be a string".to_string(),
                            span: node.span,
                        });
                    };
                    let bare_event_player = self
                        .program
                        .values
                        .get(args[0])
                        .is_some_and(|node| matches!(node.value, wir::Value::EventPlayer));
                    if !bare_event_player {
                        out.push('(');
                    }
                    self.value(args[0], out)?;
                    if !bare_event_player {
                        out.push(')');
                    }
                    write!(out, ".{member}").unwrap();
                    if let Some(index) = args.get(2) {
                        out.push('[');
                        self.value(*index, out)?;
                        out.push(']');
                    }
                    return Ok(());
                }
                if is_comparison_operator(name) {
                    // Canonical form: Compare(a, op, b).
                    if args.len() != 2 {
                        return Err(WorkshopError::Malformed {
                            message: format!("comparison call '{name}' must have 2 args"),
                            span: None,
                        });
                    }
                    out.push_str(&self.spelling(Kind::Value, "compare")?);
                    out.push('(');
                    self.value(args[0], out)?;
                    write!(out, ", {name}, ").unwrap();
                    self.value(args[1], out)?;
                    out.push(')');
                    return Ok(());
                }
                // Unary minus renders as Multiply(-1, x); the reference folds
                // literal negation, handled by the compat constant-fold pass.
                if name == "-" && args.len() == 1 {
                    out.push_str(&self.spelling(Kind::Value, "multiply")?);
                    out.push_str("(-1, ");
                    self.value(args[0], out)?;
                    out.push(')');
                    return Ok(());
                }
                // `getAllPlayers()` is OverPy's All Players(All Teams).
                if name == "getAllPlayers" && args.is_empty() {
                    out.push_str(&self.spelling(Kind::Value, "allPlayers")?);
                    out.push('(');
                    out.push_str(&self.enum_spelling("Team", "ALL")?);
                    out.push(')');
                    return Ok(());
                }
                // Binary arithmetic operators and native `.opy` source names
                // map to canonical catalog ids at emission (presentation
                // concern; the compat pass folds constants to match the
                // reference exactly).
                let canonical = match name.as_str() {
                    "+" => Some("add"),
                    "-" => Some("subtract"),
                    "*" => Some("multiply"),
                    "/" => Some("divide"),
                    "len" => Some("countOf"),
                    "abs" => Some("absoluteValue"),
                    "sqrt" => Some("squareRoot"),
                    "createBeam" => Some("createBeamEffect"),
                    "random.uniform" => Some("randomReal"),
                    "random.choice" => Some("randomValueInArray"),
                    "format" => Some("customString"),
                    _ => None,
                };
                let spelling = if let Some(canonical) = canonical {
                    self.spelling(Kind::Value, canonical)?
                } else {
                    self.spelling(Kind::Value, name)?
                };
                // `format` (frontend) and `customString` (parsed ws text) are
                // the same node.
                let is_custom_string = canonical == Some("customString") || name == "customString";
                if name == "string" {
                    out.push_str(&spelling);
                    out.push('(');
                    if let Some(first) = args.first() {
                        self.localized_string_value(*first, out)?;
                        if args.len() > 1 {
                            out.push_str(", ");
                            self.args(&args[1..], out)?;
                        }
                    }
                    out.push(')');
                } else if args.is_empty() {
                    // Constants (e.g. Empty Array) emit as bare spellings.
                    out.push_str(&spelling);
                } else if is_custom_string {
                    // `.format()` calls canonicalize: constant numeric
                    // arguments fold into the substituted text, implicit
                    // `{}` placeholders renumber to the oracle's explicit
                    // form, and remaining variable arguments wrap (the
                    // oracle spelling, #87). The canonical text feeds the
                    // value-string path (re-escaping/splitting) when no
                    // arguments remain.
                    match self.canonicalize_format_call(args)? {
                        Some((text, variable_args)) => {
                            if variable_args.is_empty() {
                                self.emit_string_value(&text, out)?;
                            } else {
                                out.push_str(&spelling);
                                out.push('(');
                                write!(out, "\"{}\"", escape_value_string(&text)).unwrap();
                                if !variable_args.is_empty() {
                                    out.push_str(", ");
                                }
                                self.args(&variable_args, out)?;
                                out.push(')');
                            }
                        }
                        None => {
                            // The `Custom String` text argument stays bare
                            // (the oracle spelling); the remaining arguments
                            // are values and wrap (#87).
                            out.push_str(&spelling);
                            out.push('(');
                            self.bare_string_value(args[0], out)?;
                            if args.len() > 1 {
                                out.push_str(", ");
                            }
                            self.args(&args[1..], out)?;
                            out.push(')');
                        }
                    }
                } else {
                    out.push_str(&spelling);
                    out.push('(');
                    self.args(args, out)?;
                    out.push(')');
                }
            }
        }
        Ok(())
    }

    pub(crate) fn localized_string_value(
        &mut self,
        id: wir::ValueId,
        out: &mut String,
    ) -> Result<()> {
        let Some(node) = self.program.values.get(id) else {
            return Err(WorkshopError::Malformed {
                message: format!("dangling value {id}"),
                span: None,
            });
        };
        let wir::Value::LocalizedString(id) = &node.value else {
            return Err(WorkshopError::Unsupported {
                message: "value 'string' argument 1 must be localized string text".to_string(),
                span: node.span,
            });
        };
        let spelling = self.localized_string_spelling(id)?;
        write!(out, "\"{}\"", escape_value_string(&spelling)).unwrap();
        Ok(())
    }

    /// Fold a `Custom String` call whose text argument and constant numeric
    /// arguments are all literals into the substituted text (the oracle's
    /// Canonicalize a `Custom String`/`.format()` call (#87): constant
    /// numeric arguments fold into the substituted text (the oracle's
    /// spelling), implicit `{}` placeholders renumber positionally to the
    /// explicit `{N}` form, and the remaining variable arguments are
    /// returned in placeholder order. Returns `None` (rendered unchanged)
    /// when nothing canonicalizes: explicit-only texts without constants,
    /// texts mixing implicit and explicit placeholders (the oracle rejects
    /// those), out-of-range placeholders, or non-String text arguments.
    pub(crate) fn canonicalize_format_call(
        &self,
        args: &[wir::ValueId],
    ) -> Result<Option<(String, Vec<wir::ValueId>)>> {
        if args.len() < 2 {
            return Ok(None);
        }
        let Some(text) = self.program.values.get(args[0]) else {
            return Ok(None);
        };
        let wir::Value::String(text) = &text.value else {
            return Ok(None);
        };
        let format_args = &args[1..];
        // Classify the placeholders: implicit `{}` consumes the next
        // argument, explicit `{N}` references argument N.
        let mut has_implicit = false;
        let mut has_explicit = false;
        let mut out_of_range = false;
        let mut cursor = 0usize;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut inner = String::new();
                let mut closed = false;
                for next in chars.by_ref() {
                    if next == '}' {
                        closed = true;
                        break;
                    }
                    inner.push(next);
                }
                if !closed {
                    break; // unterminated brace: literal text
                }
                if inner.is_empty() {
                    if cursor >= format_args.len() {
                        out_of_range = true;
                    }
                    cursor += 1;
                    has_implicit = true;
                } else if inner.chars().all(|c| c.is_ascii_digit()) {
                    match inner.parse::<usize>() {
                        Ok(index) if index < format_args.len() => has_explicit = true,
                        _ => out_of_range = true,
                    }
                } else {
                    out_of_range = true;
                }
            }
        }
        if out_of_range || (has_implicit && has_explicit) {
            return Ok(None);
        }
        let mut any_constant = false;
        for id in format_args {
            let Some(node) = self.program.values.get(*id) else {
                return Ok(None);
            };
            if matches!(node.value, wir::Value::Number { .. }) {
                any_constant = true;
            }
        }
        if !has_implicit && !any_constant {
            return Ok(None);
        }
        // Canonicalize: fold constants inline at their placeholder, renumber
        // variable placeholders positionally, keep variable arguments in
        // placeholder order.
        let mut canonical = String::with_capacity(text.len());
        let mut variable_args = Vec::new();
        let mut variable_index = 0usize;
        let mut cursor = 0usize;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut inner = String::new();
                let mut closed = false;
                for next in chars.by_ref() {
                    if next == '}' {
                        closed = true;
                        break;
                    }
                    inner.push(next);
                }
                if !closed {
                    canonical.push('{');
                    canonical.push_str(&inner);
                    break;
                }
                let index = if inner.is_empty() {
                    let index = cursor;
                    cursor += 1;
                    index
                } else {
                    match inner.parse::<usize>() {
                        Ok(index) => index,
                        Err(_) => {
                            canonical.push('{');
                            canonical.push_str(&inner);
                            canonical.push('}');
                            continue;
                        }
                    }
                };
                let Some(arg) = format_args.get(index).copied() else {
                    canonical.push('{');
                    canonical.push_str(&inner);
                    canonical.push('}');
                    continue;
                };
                let node = self.program.values.get(arg);
                if let Some(wir::Value::Number { value, .. }) = node.map(|node| &node.value) {
                    canonical.push_str(&fold_number(*value));
                } else {
                    write!(canonical, "{{{variable_index}}}").unwrap();
                    variable_index += 1;
                    variable_args.push(arg);
                }
            } else {
                canonical.push(ch);
            }
        }
        Ok(Some((canonical, variable_args)))
    }

    /// The localized spelling of a modify operator, resolved through the
    /// catalog (fallback-aware).
    pub(crate) fn modify_op_spelling(&mut self, op: wir::ModifyOp) -> Result<String> {
        self.spelling(Kind::Operator, op.catalog_id())
    }

    /// The localized spelling of a canonical builtin id, resolving through
    /// the catalog: a dangling id is `Unknown`, an id without a target-locale
    /// mapping is `MissingMapping` unless an opt-in fallback locale declares
    /// one (recorded in [`EmitContext::fallback_ids`]).
    pub(crate) fn spelling(&mut self, kind: Kind, id: &str) -> Result<String> {
        let Some(entry) = self.catalog.entry(kind, id) else {
            return Err(WorkshopError::Unknown {
                kind: kind.as_str(),
                spelling: id.to_string(),
                locale: self.locale.clone(),
                span: None,
            });
        };
        if let Some(spelling) = entry.spelling(&self.locale) {
            return Ok(spelling.to_string());
        }
        if let Some(fallback) = &self.fallback {
            if let Some(spelling) = entry.spelling(fallback) {
                self.fallback_ids.push(id.to_string());
                return Ok(spelling.to_string());
            }
        }
        Err(WorkshopError::MissingMapping {
            kind: kind.as_str(),
            id: id.to_string(),
            locale: self.locale.clone(),
        })
    }

    pub(crate) fn localized_string_spelling(&mut self, id: &str) -> Result<String> {
        if let Some(spelling) = self.catalog.localized_string_spelling(&self.locale, id) {
            return Ok(spelling.to_string());
        }
        if let Some(fallback) = &self.fallback {
            if let Some(spelling) = self.catalog.localized_string_spelling(fallback, id) {
                self.fallback_ids.push(format!("localizedString.{id}"));
                return Ok(spelling.to_string());
            }
        }
        if self.catalog.localized_strings().any(|entry| entry.id == id) {
            return Err(WorkshopError::MissingMapping {
                kind: "localized string",
                id: id.to_string(),
                locale: self.locale.clone(),
            });
        }
        Err(WorkshopError::Unknown {
            kind: "localized string",
            spelling: id.to_string(),
            locale: self.locale.clone(),
            span: None,
        })
    }

    pub(crate) fn structural(&mut self, id: &str) -> Result<String> {
        self.spelling(Kind::Structural, id)
    }

    /// The localized spelling of a canonical enum member, resolving through
    /// the catalog (fallback-aware; see [`EmitContext::spelling`]).
    pub(crate) fn enum_spelling(&mut self, domain: &str, member: &str) -> Result<String> {
        let Some(domain_entry) = self.catalog.enum_domain(domain) else {
            return Err(WorkshopError::Unknown {
                kind: "enum domain",
                spelling: domain.to_string(),
                locale: self.locale.clone(),
                span: None,
            });
        };
        let Some(member_entry) = domain_entry.members.iter().find(|m| m.member == member) else {
            return Err(WorkshopError::Unknown {
                kind: "enum member",
                spelling: format!("{domain}.{member}"),
                locale: self.locale.clone(),
                span: None,
            });
        };
        if let Some(spelling) = member_entry.spelling(&self.locale) {
            return Ok(spelling.to_string());
        }
        if let Some(fallback) = &self.fallback {
            if let Some(spelling) = member_entry.spelling(fallback) {
                self.fallback_ids.push(format!("{domain}.{member}"));
                return Ok(spelling.to_string());
            }
        }
        Err(WorkshopError::MissingMapping {
            kind: "enum member",
            id: format!("{domain}.{member}"),
            locale: self.locale.clone(),
        })
    }

    /// Render a value that must stay a bare string (the `Custom String` text
    /// argument). Any non-string value falls back to the normal renderer.
    pub(crate) fn bare_string_value(&mut self, id: wir::ValueId, out: &mut String) -> Result<()> {
        let Some(node) = self.program.values.get(id) else {
            return Err(WorkshopError::Malformed {
                message: format!("dangling value {id}"),
                span: None,
            });
        };
        if let wir::Value::String(value) = &node.value {
            write!(out, "\"{}\"", escape_value_string(value)).unwrap();
            return Ok(());
        }
        self.value(id, out)
    }

    /// Emit a value-position string as `Custom String("...")`, splitting it
    /// into a continuation chain when it exceeds the Workshop 128-char limit.
    pub(crate) fn emit_string_value(&mut self, value: &str, out: &mut String) -> Result<()> {
        let spelling = self.spelling(Kind::Value, "customString")?;
        let segments = split_string(value);
        emit_string_chain(&spelling, &segments, out);
        Ok(())
    }
}
