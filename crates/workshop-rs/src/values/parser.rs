use crate::catalog::CatalogEntry;
use crate::frontend::parser::*;

impl ParseContext<'_> {
    pub(crate) fn resolve_enum_domain_mixed(&self, spelling: &str) -> Option<&str> {
        self.or_primary(|locale| self.catalog.resolve_enum_domain(locale, spelling))
    }

    pub(crate) fn resolve_enum_member_mixed(
        &self,
        domain: &str,
        spelling: &str,
    ) -> Option<(String, String)> {
        let alternate = (!spelling.contains(": ") && spelling.contains(':'))
            .then(|| spelling.replacen(':', ": ", 1));
        self.catalog
            .resolve_enum_member(domain, &self.locale, spelling)
            .or_else(|| {
                alternate.as_deref().and_then(|spelling| {
                    self.catalog
                        .resolve_enum_member(domain, &self.locale, spelling)
                })
            })
            .or_else(|| {
                self.or_primary(|locale| {
                    self.catalog.resolve_enum_member(
                        domain,
                        locale,
                        alternate.as_deref().unwrap_or(spelling),
                    )
                })
            })
    }

    fn claim_condition_pending(&mut self) -> bool {
        std::mem::replace(
            &mut self.in_condition,
            std::mem::take(&mut self.condition_pending),
        )
    }

    pub(crate) fn value(&mut self) -> Result<wir::ValueId> {
        let saved = self.claim_condition_pending();
        let value_start = self.peek().map(|token| token.start);
        let mut value = self.primary()?;
        loop {
            if let Some(Token {
                kind: TokenKind::LBracket,
                start,
                ..
            }) = self.peek()
            {
                self.pos += 1;
                let index = self.value()?;
                let end = self.peek().map(|token| token.end).unwrap_or(start);
                self.expect(TokenKind::RBracket, "expected ']' after array index")?;
                value = self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: "valueInArray".to_string(),
                        args: vec![value, index],
                    },
                    Some(Span::new(self.file(), start, end)),
                ));
                continue;
            }
            if matches!(
                self.peek(),
                Some(Token {
                    kind: TokenKind::Dot,
                    ..
                })
            ) {
                self.pos += 1;
                let (name, name_start, name_end) = self.phrase()?;
                value = self.member_access(value, name, name_start, name_end)?;
                continue;
            }
            if let Some(Token {
                kind: TokenKind::Op(op),
                start,
                end,
            }) = self.peek()
            {
                if op == "?" {
                    self.pos += 1;
                    let when_true = self.value()?;
                    self.expect(TokenKind::Colon, "expected ':' in conditional value")?;
                    let when_false = self.value()?;
                    let (extent_start, extent_end) = (
                        self.extent_or_span(value).map(|span| span.start),
                        self.extent_or_span(when_false).map(|span| span.end),
                    );
                    let call = self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: "ifThenElse".to_string(),
                            args: vec![value, when_true, when_false],
                        },
                        Some(Span::new(self.file(), start, end)),
                    ));
                    value = match (extent_start, extent_end) {
                        (Some(start), Some(end)) => self.extend_extent(call, start, end),
                        _ => call,
                    };
                    continue;
                }
                let compound_assignment = matches!(
                    self.peek_kind_at(1),
                    Some(TokenKind::Op(equal)) if equal == "="
                );
                if !compound_assignment
                    && (wir::is_comparison_operator(&op)
                        || matches!(op.as_str(), "and" | "or" | "+" | "-" | "*" | "/" | "%"))
                {
                    self.pos += 1;
                    let saved_condition = std::mem::replace(&mut self.in_condition, false);
                    let right = self.primary();
                    self.in_condition = saved_condition;
                    let right = right?;
                    let name = match op.as_str() {
                        "+" => "add",
                        "-" => "subtract",
                        "*" => "multiply",
                        "/" => "divide",
                        "%" => "modulo",
                        _ => op.as_str(),
                    };
                    let (extent_start, extent_end) = (
                        self.extent_or_span(value).map(|span| span.start),
                        self.extent_or_span(right).map(|span| span.end),
                    );
                    let call = self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: name.to_string(),
                            args: vec![value, right],
                        },
                        Some(Span::new(self.file(), start, end)),
                    ));
                    value = match (extent_start, extent_end) {
                        (Some(start), Some(end)) => self.extend_extent(call, start, end),
                        _ => call,
                    };
                    continue;
                }
            }
            break;
        }
        self.in_condition = saved;
        if let Some(start) = value_start {
            self.extend_extent(value, start, self.previous_span().1);
        }
        Ok(value)
    }

    pub(crate) fn catalog_value_spelling(&mut self) -> Option<(String, Position, Position)> {
        let catalog = self.catalog;
        let locale = self.locale.clone();
        let primary = catalog.primary_locale();
        self.catalog_spelling(|spelling| {
            catalog.resolve(Kind::Value, &locale, spelling).is_some()
                || (locale != *primary && catalog.resolve(Kind::Value, primary, spelling).is_some())
                || !catalog.bare_member_matches(&locale, spelling).is_empty()
        })
    }

    pub(super) fn catalog_value_call_or_member(
        &mut self,
        phrase: String,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        if matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::LParen,
                ..
            })
        ) {
            self.call_or_enum(&phrase, start, end)
        } else {
            self.bare_member_resolved(&phrase, start, end)
        }
    }

    pub(crate) fn call_or_enum(
        &mut self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        let prefer_enum = self.catalog.enum_domain("Hero").is_some()
            && (canonical_keyword(phrase) == "Hero"
                || self.resolve_enum_domain_mixed(phrase) == Some("Hero"));
        if !prefer_enum && let Some(entry) = self.resolve_entry(Kind::Value, phrase) {
            return self.value_call(entry, phrase, start, end);
        }
        let action_error = self.action_in_value_error(phrase, start, end);
        if let Some(domain_name) = self
            .resolve_enum_domain_mixed(phrase)
            .or_else(|| self.resolve_enum_domain_mixed(canonical_keyword(phrase)))
        {
            let domain = self
                .catalog
                .enum_domain(domain_name)
                .expect("resolved enum domain must exist");
            let saved = self.pos;
            return match (self.enum_call(domain, start, end), action_error) {
                (Ok(value), _) => Ok(value),
                (Err(_), Some(error)) => Err(error),
                (Err(enum_error), None) => {
                    if prefer_enum {
                        self.pos = saved;
                        if let Some(entry) = self.resolve_entry(Kind::Value, phrase)
                            && let Ok(value) = self.value_call(entry, phrase, start, end)
                        {
                            return Ok(value);
                        }
                        self.pos = saved;
                    }
                    Err(enum_error)
                }
            };
        }
        if let Some(error) = action_error {
            return Err(error);
        }
        if matches!(self.peek_kind_at(0), Some(TokenKind::LParen)) {
            self.pos += 1;
            self.call_stack.push(phrase.to_string());
            let args = self.opaque_value_args()?;
            self.call_stack.pop();
            self.expect(TokenKind::RParen, "expected ')' after value call")?;
            return Ok(self.push(
                ValueNode::new(
                    Value::Call {
                        name: phrase.to_string(),
                        args,
                    },
                    Some(Span::new(self.file(), start, end)),
                ),
                start,
            ));
        }
        Ok(self.push(
            ValueNode::new(
                Value::Call {
                    name: phrase.to_string(),
                    args: Vec::new(),
                },
                Some(Span::new(self.file(), start, end)),
            ),
            start,
        ))
    }

    fn value_call(
        &mut self,
        entry: &CatalogEntry,
        _phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        self.expect(TokenKind::LParen, "expected '(' after value name")?;
        if entry.id == "compare" {
            let left = self.value_in_domain(None)?;
            self.expect(TokenKind::Comma, "expected ',' after Compare operand")?;
            let (op, op_start, op_end) = match self.next() {
                Some(Token {
                    kind: TokenKind::Op(op),
                    start,
                    end,
                }) => (op, start, end),
                Some(token) => {
                    return Err(self.malformed("expected a comparison operator in Compare", &token));
                }
                None => {
                    return Err(
                        self.malformed("expected a comparison operator in Compare", self.eof())
                    );
                }
            };
            self.expect(TokenKind::Comma, "expected ',' after Compare operator")?;
            let right = self.value_in_domain(None)?;
            self.expect(TokenKind::RParen, "expected ')'")?;
            let span = Some(Span::new(self.file(), op_start, op_end));
            return Ok(self.push(
                ValueNode::new(
                    Value::Call {
                        name: op,
                        args: vec![left, right],
                    },
                    span,
                ),
                start,
            ));
        }
        let args = self.value_args(entry.id.as_str())?;
        self.expect(TokenKind::RParen, "expected ')'")?;
        Ok(self.push(
            ValueNode::new(
                Value::Call {
                    name: entry.id.clone(),
                    args,
                },
                Some(Span::new(self.file(), start, end)),
            ),
            start,
        ))
    }

    fn enum_call(
        &mut self,
        domain: &crate::catalog::EnumDomain,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        self.expect(TokenKind::LParen, "expected '('")?;
        let (member_phrase, _, _) = self.enum_member_phrase()?;
        let member = self
            .resolve_enum_member_mixed(&domain.domain, &member_phrase)
            .unwrap_or_else(|| unresolved_enum_member(&domain.domain, &member_phrase));
        self.expect(TokenKind::RParen, "expected ')' after enum member")?;
        Ok(self.push(
            ValueNode::new(
                Value::Enum {
                    value_type: member.0,
                    value: member.1,
                },
                Some(Span::new(self.file(), start, end)),
            ),
            start,
        ))
    }

    fn action_in_value_error(
        &self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Option<WorkshopError> {
        let is_action = self.resolve_entry(Kind::Action, phrase).is_some()
            || self
                .resolve_entry(Kind::Action, &format!("{phrase} "))
                .is_some()
            || self
                .resolve_entry(Kind::Structural, phrase)
                .is_some_and(|entry| {
                    matches!(
                        entry.id.as_str(),
                        "setGlobalVariable"
                            | "modifyGlobalVariable"
                            | "setPlayerVariable"
                            | "modifyPlayerVariable"
                            | "callSubroutine"
                            | "forGlobalVariable"
                            | "forPlayerVariable"
                            | "if"
                            | "while"
                    )
                });
        is_action.then(|| {
            WorkshopError::malformed(
                format!(
                    "action '{phrase}' cannot be used as a {}",
                    if self.in_condition {
                        "condition"
                    } else {
                        "value"
                    }
                ),
                Some(Span::new(self.file(), start, self.call_site_end(end))),
            )
        })
    }

    fn call_site_end(&self, end: Position) -> Position {
        if !matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::LParen,
                ..
            })
        ) {
            return end;
        }
        let mut depth = 0usize;
        for token in &self.tokens[self.pos..] {
            match token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return token.end;
                    }
                }
                _ => {}
            }
        }
        end
    }

    pub(crate) fn bare_member(
        &mut self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        if !matches!(phrase, "None" | "无" | "True" | "真" | "False" | "假")
            && matches!(self.peek_kind_at(0), Some(TokenKind::Colon))
        {
            let saved = self.pos;
            self.pos += 1;
            let (suffix, _, suffix_end) = self.phrase()?;
            for joined in [
                format!("{phrase}: {suffix}"),
                format!("{phrase} : {suffix}"),
                format!("{phrase}:{suffix}"),
                format!("{phrase} :{suffix}"),
            ] {
                let recognized = self
                    .expected_domain
                    .and_then(|domain| self.resolve_enum_member_mixed(domain, &joined))
                    .or_else(|| self.resolve_enum_member_mixed("Hero", &joined))
                    .is_some()
                    || self
                        .catalog
                        .bare_member_matches(&self.locale, &joined)
                        .iter()
                        .any(|(domain, _)| domain != "EventTeam" && domain != "EventPlayer");
                if recognized {
                    return self.bare_member_resolved(&joined, start, suffix_end);
                }
            }
            self.pos = saved;
        }
        self.bare_member_resolved(phrase, start, end)
    }

    /// An `EventPlayer` value just consumed; wraps it in a `PlayerVariable`
    /// when a `.name` member access follows.
    pub(super) fn event_player_value(&mut self, start: Position) -> Result<wir::ValueId> {
        let player = self.target.values.push(ValueNode::new(
            Value::EventPlayer,
            Some(Span::new(self.file(), start, self.previous_span().1)),
        ));
        if !matches!(self.peek_kind_at(0), Some(TokenKind::Dot)) {
            return Ok(player);
        }
        self.pos += 1;
        let (name, name_start, name_end) = self.phrase()?;
        let variable = self.player_by_name(&name)?;
        let span = Some(Span::new(self.file(), name_start, name_end));
        Ok(self.push_extended(
            ValueNode::new(Value::PlayerVariable { player, variable }, span)
                .with_identifier(Some(Span::new(self.file(), name_start, name_end))),
            Span::new(self.file(), start, name_end),
        ))
    }

    fn push_enum(
        &mut self,
        value_type: String,
        value: String,
        start: Position,
        end: Position,
    ) -> wir::ValueId {
        self.target.values.push(ValueNode::new(
            Value::Enum { value_type, value },
            Some(Span::new(self.file(), start, end)),
        ))
    }

    pub(crate) fn bare_member_resolved(
        &mut self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        match (matches!(phrase, "None" | "无"), self.expected_domain) {
            (true, Some(expected))
                if matches!(
                    expected,
                    "ChaseTimeReeval"
                        | "ChaseRateReeval"
                        | "Invis"
                        | "ThrottleReeval"
                        | "EffectReeval"
                ) =>
            {
                return Ok(self.push_enum(expected.to_string(), "NONE".to_string(), start, end));
            }
            _ => {}
        }
        if let Some(variable) = self.globals.get(phrase).copied() {
            return Ok(self.push(
                ValueNode::new(
                    Value::GlobalVariable(variable),
                    Some(Span::new(self.file(), start, end)),
                )
                .with_identifier(Some(Span::new(self.file(), start, end))),
                start,
            ));
        }
        let member = self
            .expected_domain
            .filter(|domain| *domain == "Color")
            .and_then(|_| self.resolve_enum_member_mixed("Team", phrase))
            .filter(|(_, member)| {
                self.catalog
                    .enum_spelling("Color", self.catalog.primary_locale(), member)
                    .is_some()
            })
            .or_else(|| {
                self.expected_domain
                    .and_then(|domain| self.resolve_enum_member_mixed(domain, phrase))
            })
            .or_else(|| self.resolve_enum_member_mixed("Team", phrase));
        if let Some((value_type, value)) = member {
            return Ok(self.push_enum(value_type, value, start, end));
        }
        if self
            .call_stack
            .last()
            .is_some_and(|call| call == "createHudText")
            && let Some((value_type, value)) = self.resolve_enum_member_mixed("HudPosition", phrase)
        {
            return Ok(self.push_enum(value_type, value, start, end));
        }
        if matches!(self.expected_domain, Some("Position"))
            && let Some((value_type, value)) = self.resolve_enum_member_mixed("Vector", phrase)
        {
            return Ok(self.push_enum(value_type, value, start, end));
        }
        let mut matches: Vec<(String, String)> = self
            .catalog
            .bare_member_matches(&self.locale, phrase)
            .into_iter()
            .filter(|(domain, _)| domain != "EventTeam" && domain != "EventPlayer")
            .collect();
        if matches.len() == 1 {
            let (value_type, value) = matches.swap_remove(0);
            return Ok(self.push_enum(value_type, value, start, end));
        }
        if matches.len() > 1 {
            if let Some(expected) = self.expected_domain {
                let pinned: Vec<&(String, String)> = matches
                    .iter()
                    .filter(|(domain, _)| domain == expected)
                    .collect();
                if pinned.len() == 1 {
                    let (value_type, value) = pinned[0].clone();
                    return Ok(self.push_enum(value_type, value, start, end));
                }
            }
            let spelling = self.target.values.push(ValueNode::new(
                Value::String(phrase.to_string()),
                Some(Span::new(self.file(), start, end)),
            ));
            let candidates = matches
                .into_iter()
                .map(|(value_type, value)| self.push_enum(value_type, value, start, end))
                .collect();
            let candidates = self
                .target
                .values
                .push(ValueNode::new(Value::Array(candidates), None));
            return Ok(self.push(
                ValueNode::new(
                    Value::Call {
                        name: crate::wir::AMBIGUOUS_ENUM_CALL.to_string(),
                        args: vec![spelling, candidates],
                    },
                    Some(Span::new(self.file(), start, end)),
                ),
                start,
            ));
        }
        if let Some(entry) = self.resolve_entry(Kind::Value, phrase) {
            if entry.id == "null" {
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Null,
                    Some(Span::new(self.file(), start, end)),
                )));
            }
            if entry.id == "eventPlayer" {
                return Ok(self.target.values.push(ValueNode::new(
                    Value::EventPlayer,
                    Some(Span::new(self.file(), start, end)),
                )));
            }
            return Ok(self.push(
                ValueNode::new(
                    Value::Call {
                        name: entry.id.clone(),
                        args: Vec::new(),
                    },
                    Some(Span::new(self.file(), start, end)),
                ),
                start,
            ));
        }
        if let Some(error) = self.action_in_value_error(phrase, start, end) {
            return Err(error);
        }
        Ok(self.push(
            ValueNode::new(
                Value::Call {
                    name: phrase.to_string(),
                    args: Vec::new(),
                },
                Some(Span::new(self.file(), start, end)),
            ),
            start,
        ))
    }

    pub(crate) fn value_args(&mut self, call_id: &str) -> Result<Vec<wir::ValueId>> {
        let mut args = Vec::new();
        if let Some(Token {
            kind: TokenKind::RParen,
            ..
        }) = self.peek()
        {
            return Ok(args);
        }
        self.call_stack.push(call_id.to_string());
        let mut arg_index = 0usize;
        let mut raw_modify_operator = false;
        loop {
            if call_id == "string" && arg_index == 0 {
                args.push(self.localized_string_argument()?);
            } else if matches!(
                call_id,
                "setGlobalVariableAtIndex" | "modifyGlobalVariableAtIndex"
            ) && arg_index == 0
            {
                let (name, start, end) = self.phrase()?;
                let variable = self.global_by_name(&name)?;
                args.push(
                    self.target.values.push(
                        ValueNode::new(
                            Value::GlobalVariable(variable),
                            Some(Span::new(self.file(), start, end)),
                        )
                        .with_identifier(Some(Span::new(
                            self.file(),
                            start,
                            end,
                        ))),
                    ),
                );
            } else if matches!(
                call_id,
                "setPlayerVariableAtIndex" | "modifyPlayerVariableAtIndex"
            ) && arg_index == 1
            {
                let saved = self.pos;
                let (name, start, end) = self.phrase()?;
                if let Some(variable) = self.players.get(&name).copied() {
                    let player = args[0];
                    args[0] = self.target.values.push(
                        ValueNode::new(
                            Value::PlayerVariable { player, variable },
                            Some(Span::new(self.file(), start, end)),
                        )
                        .with_identifier(Some(Span::new(
                            self.file(),
                            start,
                            end,
                        ))),
                    );
                } else {
                    self.pos = saved;
                    let domain = self.context.expected_domain(call_id, arg_index);
                    args.push(self.value_in_domain(domain)?);
                }
            } else if matches!(
                call_id,
                "modifyGlobalVariableAtIndex" | "modifyPlayerVariableAtIndex"
            ) && {
                let operator_position = arg_index == 2 || arg_index == 3 && raw_modify_operator;
                let saved = self.pos;
                let is_operator = operator_position && self.modify_op().is_ok();
                self.pos = saved;
                is_operator
            } {
                let operator = self.modify_op()?;
                let name = operator.catalog_id();
                args.push(self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: name.to_string(),
                        args: Vec::new(),
                    },
                    None,
                )));
            } else {
                if matches!(
                    call_id,
                    "modifyGlobalVariableAtIndex" | "modifyPlayerVariableAtIndex"
                ) && arg_index == 2
                {
                    let saved = self.pos;
                    raw_modify_operator = self.modify_op().is_err();
                    self.pos = saved;
                }
                let domain = self
                    .context
                    .expected_domain(call_id, arg_index)
                    .or_else(|| {
                        self.expected_domain
                            .filter(|_| matches!(call_id, "array" | "randomValueInArray"))
                    });
                args.push(self.value_in_domain(domain)?);
            }
            arg_index += 1;
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Comma,
                    ..
                }) => {
                    self.pos += 1;
                }
                Some(Token {
                    kind: TokenKind::Colon,
                    ..
                }) => {
                    if self.named_argument_separator()? {
                        break;
                    }
                }
                _ => break,
            }
        }
        self.call_stack.pop();
        Ok(args)
    }

    pub(crate) fn localized_string_argument(&mut self) -> Result<wir::ValueId> {
        let Some(token) = self.peek() else {
            return Err(self.malformed("expected a localized string", self.eof()));
        };
        if matches!(token.kind, TokenKind::String(_)) {
            let span = Some(Span::new(self.file(), token.start, token.end));
            let content = self.expect_string("expected a localized string")?;
            return self.localized_string_literal(content, span);
        }

        // Workshop.codes presents the preset form as `String(Hello, ...)`,
        // while independent decompilers emit quoted text. Preserve both raw
        // spellings as a localized-string literal, but leave known expressions
        // on the normal path so validation can reject them precisely.
        let saved = self.pos;
        let (phrase, _, _) = self.phrase()?;
        let expression_like = self.resolve_entry(Kind::Value, &phrase).is_some()
            || matches!(self.canonical_keyword(&phrase).as_str(), "True" | "False")
            || self
                .peek()
                .is_some_and(|token| matches!(&token.kind, TokenKind::LParen | TokenKind::Dot));
        self.pos = saved;
        if expression_like || !matches!(token.kind, TokenKind::Word(_) | TokenKind::Number { .. }) {
            return self.value();
        }

        let catalog = self.catalog;
        let locale = self.locale.clone();
        if let Some((text, start, end)) = self.catalog_spelling(|spelling| {
            catalog
                .resolve_localized_string(&locale, spelling)
                .is_some()
        }) {
            return self.localized_string_literal(text, Some(Span::new(self.file(), start, end)));
        }
        let (text, start, end) = self.phrase()?;
        let (text, end) = self
            .extend_catalog_spelling(&text, end, |spelling| {
                catalog
                    .resolve_localized_string(&locale, spelling)
                    .is_some()
            })
            .unwrap_or((text, end));
        self.localized_string_literal(text, Some(Span::new(self.file(), start, end)))
    }

    pub(crate) fn localized_string_literal(
        &mut self,
        text: String,
        span: Option<Span>,
    ) -> Result<wir::ValueId> {
        let Some(entry) = self.catalog.resolve_localized_string(&self.locale, &text) else {
            let candidates = crate::core::suggest::nearest(
                &text,
                self.catalog
                    .localized_strings()
                    .flat_map(|entry| entry.spellings(&self.locale))
                    .map(String::as_str),
                crate::core::suggest::CANDIDATE_LIMIT,
            );
            return Err(WorkshopError::unknown(
                "localized string",
                text,
                self.locale.clone(),
                span,
            )
            .with_candidates(candidates));
        };
        Ok(self.target.values.push(ValueNode::new(
            Value::LocalizedString(entry.id.clone()),
            span,
        )))
    }

    fn named_argument_separator(&mut self) -> Result<bool> {
        self.pos += 1;
        if !matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::RParen,
                ..
            })
        ) {
            let _ = self.phrase()?;
        }
        match self.peek() {
            Some(Token {
                kind: TokenKind::Comma,
                ..
            }) => {
                self.pos += 1;
                Ok(false)
            }
            Some(Token {
                kind: TokenKind::RParen,
                ..
            }) => Ok(true),
            Some(token) => Err(self.malformed(r"expected ',' or ')'", &token)),
            None => Err(self.malformed("unexpected end of value call", self.eof())),
        }
    }

    pub(crate) fn opaque_value_args(&mut self) -> Result<Vec<wir::ValueId>> {
        let mut args = Vec::new();
        if matches!(self.peek_kind_at(0), Some(TokenKind::RParen)) {
            return Ok(args);
        }
        loop {
            args.push(self.value()?);
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Colon,
                    ..
                }) => {
                    if self.named_argument_separator()? {
                        break;
                    }
                }
                Some(Token {
                    kind: TokenKind::Comma,
                    ..
                }) => self.pos += 1,
                Some(Token {
                    kind: TokenKind::RParen,
                    ..
                }) => break,
                Some(token) => return Err(self.malformed("expected ',' or ')'", &token)),
                None => return Err(self.malformed("unexpected end of value call", self.eof())),
            }
        }
        Ok(args)
    }

    pub(crate) fn push_bool(
        &mut self,
        value: bool,
        start: Position,
        end: Position,
    ) -> wir::ValueId {
        self.push(
            ValueNode::new(Value::Bool(value), Some(Span::new(self.file(), start, end))),
            start,
        )
    }
    pub(super) fn member_access(
        &mut self,
        receiver: wir::ValueId,
        name: String,
        name_start: Position,
        name_end: Position,
    ) -> Result<wir::ValueId> {
        if matches!(
            self.target.values.get(receiver),
            Some(ValueNode {
                value: Value::EventPlayer,
                ..
            })
        ) && self.players.contains_key(&name)
        {
            let variable = self.player_by_name(&name)?;
            let extent = self
                .extent_or_span(receiver)
                .map(|span| Span::new(self.file(), span.start, name_end));
            return Ok(self.push_extended(
                ValueNode::new(
                    Value::PlayerVariable {
                        player: receiver,
                        variable,
                    },
                    None,
                ),
                extent.unwrap_or_else(|| Span::new(self.file(), name_start, name_end)),
            ));
        }
        let member = self
            .target
            .values
            .push(ValueNode::new(Value::String(name), None));
        let mut args = vec![receiver, member];
        if matches!(
            self.peek(),
            Some(Token {
                kind: TokenKind::LBracket,
                ..
            })
        ) {
            self.pos += 1;
            args.push(self.value()?);
            self.expect(TokenKind::RBracket, "expected ']' after member index")?;
        }
        let end = self.previous_span().1;
        let extent = self
            .extent_or_span(receiver)
            .map(|span| Span::new(self.file(), span.start, end));
        Ok(self.push_extended(
            ValueNode::new(
                Value::Call {
                    name: "memberAccess".to_string(),
                    args,
                },
                None,
            ),
            extent.unwrap_or_else(|| Span::new(self.file(), end, end)),
        ))
    }
}

fn unresolved_enum_member(domain: &str, spelling: &str) -> (String, String) {
    (domain.to_string(), spelling.to_string())
}
