// ParseContext behavior owned by the Workshop values domain.

use crate::frontend::parser::*;

impl ParseContext<'_> {
    pub(crate) fn contextual_coercions(
        &self,
        call_id: &str,
        arg_index: usize,
    ) -> Option<crate::catalog::ParamCoercions> {
        [Kind::Action, Kind::Value].into_iter().find_map(|kind| {
            self.catalog
                .entry(kind, call_id)
                .and_then(|entry| entry.param_coercions(arg_index))
                .copied()
        })
    }

    pub(crate) fn is_zero_number(&self, value_id: wir::ValueId) -> bool {
        matches!(
            self.target.values.get(value_id),
            Some(ValueNode {
                value: Value::Number { value, .. },
                ..
            }) if *value == 0.0
        )
    }

    pub(crate) fn is_empty_string(&self, value_id: wir::ValueId) -> bool {
        matches!(
            self.target.values.get(value_id),
            Some(ValueNode {
                value: Value::String(value),
                ..
            }) if value.is_empty()
        )
    }

    pub(crate) fn normalize_contextual_argument(
        &mut self,
        call_id: &str,
        arg_index: usize,
        value_id: wir::ValueId,
    ) -> wir::ValueId {
        let Some(coercions) = self.contextual_coercions(call_id, arg_index) else {
            return value_id;
        };
        self.normalize_value_with_coercions(coercions, value_id)
    }

    pub(crate) fn normalize_value_with_coercions(
        &mut self,
        coercions: ParamCoercions,
        value_id: wir::ValueId,
    ) -> wir::ValueId {
        let Some(node) = self.target.values.get(value_id) else {
            return value_id;
        };
        let span = node.span;
        let replacement = match &node.value {
            Value::Bool(false) if coercions.false_as_number => Some(Value::Number {
                value: 0.0,
                text: "0".to_string(),
            }),
            Value::Bool(true) if coercions.true_as_number => Some(Value::Number {
                value: 1.0,
                text: "1".to_string(),
            }),
            Value::Number { value, .. } if coercions.zero_as_null && *value == 0.0 => {
                Some(Value::Null)
            }
            Value::Vector { x, y, z }
                if coercions.null_vector_as_null
                    && self.is_zero_number(*x)
                    && self.is_zero_number(*y)
                    && self.is_zero_number(*z) =>
            {
                Some(Value::Null)
            }
            Value::Call { name, args }
                if coercions.null_vector_as_null
                    && name == "vector"
                    && args.len() == 3
                    && args.iter().all(|value_id| self.is_zero_number(*value_id)) =>
            {
                Some(Value::Null)
            }
            Value::Call { name, args }
                if coercions.empty_array_as_string && name == "emptyArray" && args.is_empty() =>
            {
                Some(Value::String(String::new()))
            }
            Value::Call { name, args }
                if coercions.empty_array_as_string
                    && name == "customString"
                    && args.len() == 1
                    && self.is_empty_string(args[0]) =>
            {
                Some(Value::String(String::new()))
            }
            Value::Array(elements) if coercions.empty_array_as_string && elements.is_empty() => {
                Some(Value::String(String::new()))
            }
            _ => None,
        };
        let Some(value) = replacement else {
            return value_id;
        };
        self.target.values.push(ValueNode::new(value, span))
    }

    pub(crate) fn normalize_modify_value(
        &mut self,
        op: ModifyOp,
        value_id: wir::ValueId,
    ) -> wir::ValueId {
        let coercions = match op {
            ModifyOp::Add
            | ModifyOp::Subtract
            | ModifyOp::Modulo
            | ModifyOp::Min
            | ModifyOp::Max
            | ModifyOp::RemoveFromArrayByIndex => ParamCoercions {
                false_as_number: true,
                true_as_number: true,
                ..Default::default()
            },
            ModifyOp::AppendToArray | ModifyOp::RemoveFromArray => ParamCoercions {
                zero_as_null: true,
                ..Default::default()
            },
            ModifyOp::Multiply | ModifyOp::Divide | ModifyOp::RaiseToPower => return value_id,
        };
        self.normalize_value_with_coercions(coercions, value_id)
    }

    pub(crate) fn modify_op_from_value(&self, value_id: wir::ValueId) -> Option<ModifyOp> {
        let Value::Call { name, args } = &self.target.values.get(value_id)?.value else {
            return None;
        };
        if !args.is_empty() {
            return None;
        }
        match name.as_str() {
            "add" => Some(ModifyOp::Add),
            "subtract" => Some(ModifyOp::Subtract),
            "multiply" => Some(ModifyOp::Multiply),
            "divide" => Some(ModifyOp::Divide),
            "modulo" => Some(ModifyOp::Modulo),
            "min" => Some(ModifyOp::Min),
            "max" => Some(ModifyOp::Max),
            "raiseToPower" => Some(ModifyOp::RaiseToPower),
            "appendToArray" => Some(ModifyOp::AppendToArray),
            "removeFromArray" | "removeFromArrayByValue" => Some(ModifyOp::RemoveFromArray),
            "removeFromArrayByIndex" => Some(ModifyOp::RemoveFromArrayByIndex),
            _ => None,
        }
    }

    pub(crate) fn normalize_modify_call_value(
        &mut self,
        call_id: &str,
        arg_index: usize,
        args: &[wir::ValueId],
        value_id: wir::ValueId,
    ) -> wir::ValueId {
        if matches!(
            call_id,
            "modifyGlobalVariableAtIndex" | "modifyPlayerVariableAtIndex"
        ) && arg_index == 3
        {
            if let Some(op) = args
                .get(2)
                .and_then(|value_id| self.modify_op_from_value(*value_id))
            {
                return self.normalize_modify_value(op, value_id);
            }
        }
        value_id
    }

    pub(crate) fn resolve_enum_domain_mixed(&self, spelling: &str) -> Option<&str> {
        self.catalog
            .resolve_enum_domain(&self.locale, spelling)
            .or_else(|| {
                if self.locale != *self.catalog.primary_locale() {
                    self.catalog
                        .resolve_enum_domain(self.catalog.primary_locale(), spelling)
                } else {
                    None
                }
            })
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
                if self.locale != *self.catalog.primary_locale() {
                    self.catalog.resolve_enum_member(
                        domain,
                        self.catalog.primary_locale(),
                        alternate.as_deref().unwrap_or(spelling),
                    )
                } else {
                    None
                }
            })
    }

    pub(crate) fn value(&mut self) -> Result<wir::ValueId> {
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
                let index = self.normalize_contextual_argument("valueInArray", 1, index);
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
                let (name, _, _) = self.phrase()?;
                let member = self
                    .target
                    .values
                    .push(ValueNode::new(Value::String(name), None));
                let mut args = vec![value, member];
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
                value = self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: "memberAccess".to_string(),
                        args,
                    },
                    None,
                ));
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
                    let when_true = self.normalize_contextual_argument("ifThenElse", 1, when_true);
                    let when_false =
                        self.normalize_contextual_argument("ifThenElse", 2, when_false);
                    value = self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: "ifThenElse".to_string(),
                            args: vec![value, when_true, when_false],
                        },
                        Some(Span::new(self.file(), start, end)),
                    ));
                    continue;
                }
                let compound_assignment = matches!(
                    self.peek_at(1).map(|token| token.kind),
                    Some(TokenKind::Op(equal)) if equal == "="
                );
                if !compound_assignment
                    && (is_comparison(&op)
                        || matches!(op.as_str(), "and" | "or" | "+" | "-" | "*" | "/" | "%"))
                {
                    self.pos += 1;
                    let right = self.primary()?;
                    let name = match op.as_str() {
                        "+" => "add",
                        "-" => "subtract",
                        "*" => "multiply",
                        "/" => "divide",
                        "%" => "modulo",
                        _ => op.as_str(),
                    };
                    let left = self.normalize_contextual_argument(name, 0, value);
                    let right = self.normalize_contextual_argument(name, 1, right);
                    value = self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: name.to_string(),
                            args: vec![left, right],
                        },
                        Some(Span::new(self.file(), start, end)),
                    ));
                    continue;
                }
            }
            break;
        }
        Ok(value)
    }

    pub(crate) fn primary(&mut self) -> Result<wir::ValueId> {
        match self.peek() {
            Some(Token {
                kind: TokenKind::Op(op),
                start,
                ..
            }) if op == "not" => {
                self.pos += 1;
                let value = self.primary()?;
                Ok(self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: "not".to_string(),
                        args: vec![value],
                    },
                    Some(Span::new(self.file(), start, self.previous_span().1)),
                )))
            }
            Some(Token {
                kind: TokenKind::Number { value, text },
                start,
                end,
            }) => {
                let span = Some(Span::new(self.file(), start, end));
                self.pos += 1;
                Ok(self
                    .target
                    .values
                    .push(ValueNode::new(Value::Number { value, text }, span)))
            }
            Some(Token {
                kind: TokenKind::Op(op),
                start,
                ..
            }) if op == "-" => {
                if let Some(Token {
                    kind: TokenKind::Number { value, text },
                    end: number_end,
                    ..
                }) = self.peek_at(1)
                {
                    let span = Some(Span::new(self.file(), start, number_end));
                    self.pos += 2;
                    Ok(self.target.values.push(ValueNode::new(
                        Value::Number {
                            value: -value,
                            text: format!("-{text}"),
                        },
                        span,
                    )))
                } else {
                    Err(self.malformed("expected a number after '-'", &self.peek().unwrap()))
                }
            }
            Some(Token {
                kind: TokenKind::String(content),
                start,
                end,
            }) => {
                let span = Some(Span::new(self.file(), start, end));
                self.pos += 1;
                Ok(self
                    .target
                    .values
                    .push(ValueNode::new(Value::String(content), span)))
            }
            Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) if matches!(canonical_keyword(&word), "Global" | "global") => {
                let (start, end) = self.span_here();
                self.pos += 1;
                // The reference's value spelling `Global Variable(name)`
                // (#119 differential evidence); the OPY form `Global.name`
                // stays supported.
                if let Some(Token {
                    kind: TokenKind::Word(next),
                    ..
                }) = self.peek()
                {
                    if canonical_keyword(&next) == "Variable"
                        || canonical_keyword(&next) == "variable"
                    {
                        self.pos += 1;
                        self.expect(TokenKind::LParen, "expected '(' after 'Global Variable'")?;
                        let (name, _, _) = self.phrase()?;
                        let variable = self.global_by_name(&name)?;
                        self.expect(TokenKind::RParen, "expected ')' after Global Variable")?;
                        let span = Some(Span::new(self.file(), start, end));
                        return Ok(self
                            .target
                            .values
                            .push(ValueNode::new(Value::GlobalVariable(variable), span)));
                    }
                }
                self.expect(TokenKind::Dot, "expected '.' after 'Global'")?;
                let (name, _, _) = self.phrase()?;
                let variable = self.global_by_name(&name)?;
                let span = Some(Span::new(self.file(), start, end));
                Ok(self
                    .target
                    .values
                    .push(ValueNode::new(Value::GlobalVariable(variable), span)))
            }
            Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) if matches!(canonical_keyword(&word), "Player" | "player")
                && matches!(
                    self.peek_at(1),
                    Some(Token {
                        kind: TokenKind::Word(next),
                        ..
                    }) if matches!(canonical_keyword(&next), "Variable" | "variable")
                ) =>
            {
                // The reference's playervar-read spelling
                // `Player Variable(player, name)` (#119 differential
                // evidence).
                let (start, end) = self.span_here();
                self.pos += 1;
                let saved = self.expected_domain;
                self.expected_domain = None;
                let result = (|| {
                    self.consume_phrase("Variable")?;
                    self.expect(TokenKind::LParen, "expected '(' after 'Player Variable'")?;
                    let player = self.value()?;
                    self.expect(TokenKind::Comma, "expected ',' after player")?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.player_by_name(&name)?;
                    self.expect(TokenKind::RParen, "expected ')' after Player Variable")?;
                    Ok(self.target.values.push(ValueNode::new(
                        Value::PlayerVariable { player, variable },
                        Some(Span::new(self.file(), start, end)),
                    )))
                })();
                self.expected_domain = saved;
                result
            }
            Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) if matches!(canonical_keyword(&word), "Event" | "event" | "Event Player") => {
                let (start, _) = self.span_here();
                self.pos += 1;
                if canonical_keyword(&word) == "Event Player" {
                    let player = self.target.values.push(ValueNode::new(
                        Value::EventPlayer,
                        Some(Span::new(self.file(), start, self.previous_span().1)),
                    ));
                    if matches!(
                        self.peek(),
                        Some(Token {
                            kind: TokenKind::Dot,
                            ..
                        })
                    ) {
                        self.pos += 1;
                        let (name, name_start, name_end) = self.phrase()?;
                        let variable = self.player_by_name(&name)?;
                        return Ok(self.target.values.push(ValueNode::new(
                            Value::PlayerVariable { player, variable },
                            Some(Span::new(self.file(), name_start, name_end)),
                        )));
                    }
                    return Ok(player);
                }
                if canonical_keyword(&word) != "Event Player"
                    && matches!(self.peek(), Some(Token { kind: TokenKind::Word(next), .. }) if matches!(canonical_keyword(&next), "Player" | "player"))
                {
                    self.pos += 1;
                    let player = self.target.values.push(ValueNode::new(
                        Value::EventPlayer,
                        Some(Span::new(self.file(), start, self.previous_span().1)),
                    ));
                    if matches!(
                        self.peek(),
                        Some(Token {
                            kind: TokenKind::Dot,
                            ..
                        })
                    ) {
                        self.pos += 1;
                        let (name, name_start, name_end) = self.phrase()?;
                        let variable = self.player_by_name(&name)?;
                        return Ok(self.target.values.push(ValueNode::new(
                            Value::PlayerVariable { player, variable },
                            Some(Span::new(self.file(), name_start, name_end)),
                        )));
                    }
                    return Ok(player);
                }
                self.pos -= 1;
                let (phrase, start, end) = self.phrase()?;
                if matches!(
                    self.peek(),
                    Some(Token {
                        kind: TokenKind::LParen,
                        ..
                    })
                ) {
                    self.call_or_enum(&phrase, start, end)
                } else {
                    self.bare_member(&phrase, start, end)
                }
            }
            Some(Token {
                kind: TokenKind::LParen,
                ..
            }) => {
                // The oracle's playervar-read spelling parenthesizes the
                // receiver: `(Event Player).p` (#87).
                self.pos += 1;
                let inner = self.value()?;
                if matches!(
                    self.peek(),
                    Some(Token {
                        kind: TokenKind::Dot,
                        ..
                    })
                ) {
                    self.pos += 1;
                    let (name, _, _) = self.phrase()?;
                    let member = self
                        .target
                        .values
                        .push(ValueNode::new(Value::String(name), None));
                    let mut args = vec![inner, member];
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
                    let mut accessed = self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: "memberAccess".to_string(),
                            args,
                        },
                        None,
                    ));
                    if matches!(self.peek(), Some(Token { kind: TokenKind::Op(op), .. }) if op == "?")
                    {
                        self.pos += 1;
                        let when_true = self.value()?;
                        self.expect(TokenKind::Colon, "expected ':' in conditional value")?;
                        let when_false = self.value()?;
                        accessed = self.target.values.push(ValueNode::new(
                            Value::Call {
                                name: "ifThenElse".to_string(),
                                args: vec![accessed, when_true, when_false],
                            },
                            None,
                        ));
                    }
                    self.expect(TokenKind::RParen, "expected ')' after member access")?;
                    return Ok(accessed);
                }
                self.expect(TokenKind::RParen, "expected ')' after parenthesized value")?;
                if let Some(Token {
                    kind: TokenKind::Dot,
                    ..
                }) = self.peek()
                {
                    self.pos += 1;
                    let (name, _, _) = self.phrase()?;
                    if matches!(
                        self.target.values.get(inner),
                        Some(ValueNode {
                            value: Value::EventPlayer,
                            ..
                        })
                    ) || self.players.contains_key(&name)
                    {
                        let variable = self
                            .players
                            .get(&name)
                            .copied()
                            .unwrap_or(self.player_by_name(&name)?);
                        Ok(self.target.values.push(ValueNode::new(
                            Value::PlayerVariable {
                                player: inner,
                                variable,
                            },
                            None,
                        )))
                    } else {
                        let member = self
                            .target
                            .values
                            .push(ValueNode::new(Value::String(name), None));
                        let mut args = vec![inner, member];
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
                        Ok(self.target.values.push(ValueNode::new(
                            Value::Call {
                                name: "memberAccess".to_string(),
                                args,
                            },
                            None,
                        )))
                    }
                } else {
                    Ok(inner)
                }
            }
            _ => {
                if let Some((phrase, start, end)) = self.catalog_phrase_with_dot() {
                    return self.bare_member_resolved(&phrase, start, end);
                }
                let (phrase, start, end) = self.phrase()?;
                match canonical_keyword(&phrase) {
                    "True" | "真" => Ok(self.push_bool(true, start, end)),
                    "False" | "假" => Ok(self.push_bool(false, start, end)),
                    "Event Player" => Ok(self.target.values.push(ValueNode::new(
                        Value::EventPlayer,
                        Some(Span::new(self.file(), start, end)),
                    ))),
                    "Null" => Ok(self.target.values.push(ValueNode::new(
                        Value::Null,
                        Some(Span::new(self.file(), start, end)),
                    ))),
                    _ => {
                        if let Some(Token {
                            kind: TokenKind::LParen,
                            ..
                        }) = self.peek()
                        {
                            self.call_or_enum(&phrase, start, end)
                        } else {
                            self.bare_member(&phrase, start, end)
                        }
                    }
                }
            }
        }
    }

    /// Consume a dotted phrase only when the complete spelling is a reviewed
    /// catalog value or enum member. Dots otherwise remain member-access
    /// syntax, and unresolved dotted identifiers keep the existing diagnostic
    /// path instead of being accepted as catalog names.
    pub(crate) fn catalog_phrase_with_dot(&mut self) -> Option<(String, Position, Position)> {
        let first = self.peek()?;
        if !matches!(first.kind, TokenKind::Word(_)) {
            return None;
        }
        let start = first.start;
        let mut end = first.end;
        let mut parts = Vec::new();
        let mut has_dot = false;
        let mut index = self.pos;
        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::Word(word) => parts.push(word.clone()),
                TokenKind::Number { text, .. } => parts.push(text.clone()),
                TokenKind::Dot => {
                    has_dot = true;
                    parts.push(".".to_string());
                }
                _ => break,
            }
            end = token.end;
            index += 1;
        }
        if !has_dot {
            return None;
        }
        let phrase = parts.join(" ").replace(" .", ".").replace(". ", ".");
        let known_value = self.resolve_entry(Kind::Value, &phrase).is_some();
        let known_member = !self
            .catalog
            .bare_member_matches(&self.locale, &phrase)
            .is_empty();
        if !known_value && !known_member {
            return None;
        }
        self.pos = index;
        Some((phrase, start, end))
    }

    pub(crate) fn call_or_enum(
        &mut self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        // A value function wins over an enum domain of the same spelling
        // (e.g. `Vector(x, y, z)` is the value function; `Vector` as an enum
        // domain only appears through bare members like `Up`).
        let prefer_enum = self.catalog.enum_domain("Hero").is_some()
            && (canonical_keyword(phrase) == "Hero"
                || self.resolve_enum_domain_mixed(phrase) == Some("Hero"));
        if !prefer_enum {
            if let Some(entry) = self.resolve_entry(Kind::Value, phrase) {
                self.expect(TokenKind::LParen, "expected '(' after value name")?;
                if entry.id == "compare" {
                    // Compare(a, op, b) -> Call(op, [a, b]). The operands are
                    // value positions, not signature-pinned arguments, so the
                    // enclosing expected domain must not leak in (#111).
                    let saved = self.expected_domain;
                    self.expected_domain = None;
                    let left = self.value();
                    self.expected_domain = saved;
                    let left = left?;
                    self.expect(TokenKind::Comma, "expected ',' after Compare operand")?;
                    let (op, op_start, op_end) = match self.next() {
                        Some(Token {
                            kind: TokenKind::Op(op),
                            start,
                            end,
                        }) => (op, start, end),
                        Some(token) => {
                            return Err(
                                self.malformed("expected a comparison operator in Compare", &token)
                            );
                        }
                        None => {
                            return Err(self.malformed(
                                "expected a comparison operator in Compare",
                                self.eof(),
                            ));
                        }
                    };
                    self.expect(TokenKind::Comma, "expected ',' after Compare operator")?;
                    let saved = self.expected_domain;
                    self.expected_domain = None;
                    let right = self.value();
                    self.expected_domain = saved;
                    let right = right?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    let span = Some(Span::new(self.file(), op_start, op_end));
                    return Ok(self.target.values.push(ValueNode::new(
                        Value::Call {
                            name: op,
                            args: vec![left, right],
                        },
                        span,
                    )));
                }
                let args = self.value_args(entry.id.as_str())?;
                self.expect(TokenKind::RParen, "expected ')'")?;
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: entry.id.clone(),
                        args,
                    },
                    Some(Span::new(self.file(), start, end)),
                )));
            }
        }
        if let Some(domain_name) = self
            .resolve_enum_domain_mixed(phrase)
            .or_else(|| self.resolve_enum_domain_mixed(canonical_keyword(phrase)))
        {
            let domain = self
                .catalog
                .enum_domain(domain_name)
                .expect("resolved enum domain must exist");
            // Enum call: `Color(Yellow)`.
            self.expect(TokenKind::LParen, "expected '('")?;
            let (member_phrase, _, _) = self.enum_member_phrase()?;
            let member = self
                .resolve_enum_member_mixed(&domain.domain, &member_phrase)
                .unwrap_or_else(|| (domain.domain.clone(), member_phrase.clone()));
            self.expect(TokenKind::RParen, "expected ')' after enum member")?;
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum {
                    value_type: member.0,
                    value: member.1,
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if matches!(self.peek().map(|token| token.kind), Some(TokenKind::LParen)) {
            self.pos += 1;
            self.call_stack.push(phrase.to_string());
            let args = self.opaque_value_args()?;
            self.call_stack.pop();
            self.expect(TokenKind::RParen, "expected ')' after value call")?;
            return Ok(self.target.values.push(ValueNode::new(
                Value::Call {
                    name: phrase.to_string(),
                    args,
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        Ok(self.target.values.push(ValueNode::new(
            Value::Call {
                name: phrase.to_string(),
                args: Vec::new(),
            },
            Some(Span::new(self.file(), start, end)),
        )))
    }

    pub(crate) fn bare_member(
        &mut self,
        phrase: &str,
        start: Position,
        end: Position,
    ) -> Result<wir::ValueId> {
        // Some enum members use a colon in their Workshop spelling (for
        // example `Arrow: Up`). `phrase()` intentionally stops at the colon
        // for ordinary identifiers, so complete the member only when the
        // enclosing signature has already declared an enum domain.
        if !matches!(phrase, "None" | "无" | "True" | "真" | "False" | "假")
            && matches!(self.peek().map(|token| token.kind), Some(TokenKind::Colon))
        {
            let saved = self.pos;
            self.pos += 1;
            let (suffix, _, suffix_end) = self.phrase()?;
            let owned_phrase = format!("{phrase}: {suffix}");
            let recognized = self
                .expected_domain
                .and_then(|domain| self.resolve_enum_member_mixed(domain, &owned_phrase))
                .or_else(|| self.resolve_enum_member_mixed("Hero", &owned_phrase));
            if recognized.is_some() {
                return self.bare_member_resolved(&owned_phrase, start, suffix_end);
            }
            self.pos = saved;
        }
        self.bare_member_resolved(phrase, start, end)
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
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Enum {
                        value_type: expected.to_string(),
                        value: "NONE".to_string(),
                    },
                    Some(Span::new(self.file(), start, end)),
                )));
            }
            _ => {}
        }
        if let Some(variable) = self.globals.get(phrase).copied() {
            return Ok(self.target.values.push(ValueNode::new(
                Value::GlobalVariable(variable),
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if let Some(expected) = self.expected_domain {
            if let Some((value_type, value)) = self.resolve_enum_member_mixed(expected, phrase) {
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Enum { value_type, value },
                    Some(Span::new(self.file(), start, end)),
                )));
            }
        }
        if let Some((value_type, value)) = self.resolve_enum_member_mixed("Team", phrase) {
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum { value_type, value },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if phrase == "Visible To and String" {
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum {
                    value_type: "HudReeval".to_string(),
                    value: "VISIBILITY_AND_STRING".to_string(),
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if self
            .call_stack
            .last()
            .is_some_and(|call| call == "createHudText")
        {
            if let Some((value_type, value)) = self.resolve_enum_member_mixed("HudPosition", phrase)
            {
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Enum { value_type, value },
                    Some(Span::new(self.file(), start, end)),
                )));
            }
        }
        if (self.expected_domain.is_none()
            && matches!(
                phrase,
                "Up" | "上" | "Down" | "下" | "Left" | "左" | "Right" | "右"
            )
            && (self
                .call_stack
                .last()
                .is_some_and(|call| matches!(call.as_str(), "multiply" | "add"))
                || self.call_stack.is_empty()
                || self
                    .call_stack
                    .iter()
                    .any(|call| call == "startAcceleration")
                || self.call_stack.iter().any(|call| {
                    call == "raycastHitPosition"
                        || call == "Direction Towards"
                        || call == "directionTowards"
                })))
            || (matches!(self.expected_domain, Some("Position")) && matches!(phrase, "Up" | "上"))
        {
            let value = match phrase {
                "Left" | "左" => "LEFT",
                "Right" | "右" => "RIGHT",
                "Down" | "下" => "DOWN",
                _ => "UP",
            };
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum {
                    value_type: "Vector".to_string(),
                    value: value.to_string(),
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        // Event filter domains are resolved by the event parser and are not
        // value-argument domains. Excluding them here keeps their spellings
        // from making unrelated bare value arguments ambiguous (for example,
        // `All` in `Set Invisible(..., All)`).
        let matches: Vec<(String, String)> = self
            .catalog
            .bare_member_matches(&self.locale, phrase)
            .into_iter()
            .filter(|(domain, _)| domain != "EventTeam" && domain != "EventPlayer")
            .collect();
        if matches.len() == 1 {
            return Ok(self.target.values.push(ValueNode::new(
                Value::Enum {
                    value_type: matches[0].0.clone(),
                    value: matches[0].1.clone(),
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        if matches.len() > 1 {
            // #111: a bare spelling shared by several enum domains resolves
            // only when the enclosing call's canonical signature pins exactly
            // one of the matching domains. No pin keeps the deterministic
            // ambiguity diagnostic — no guessing, no global precedence.
            if let Some(expected) = self.expected_domain {
                let pinned: Vec<&(String, String)> = matches
                    .iter()
                    .filter(|(domain, _)| domain == expected)
                    .collect();
                if pinned.len() == 1 {
                    return Ok(self.target.values.push(ValueNode::new(
                        Value::Enum {
                            value_type: pinned[0].0.clone(),
                            value: pinned[0].1.clone(),
                        },
                        Some(Span::new(self.file(), start, end)),
                    )));
                }
            }
            return Err(WorkshopError::Unsupported {
                message: format!("ambiguous enum member '{phrase}' (multiple domains match)"),
                span: Some(Span::new(self.file(), start, end)),
            });
        }
        // A bare value constant (e.g. Empty Array).
        if let Some(entry) = self.resolve_entry(Kind::Value, phrase) {
            if entry.id == "null" {
                return Ok(self.target.values.push(ValueNode::new(
                    Value::Null,
                    Some(Span::new(self.file(), start, end)),
                )));
            }
            return Ok(self.target.values.push(ValueNode::new(
                Value::Call {
                    name: entry.id.clone(),
                    args: Vec::new(),
                },
                Some(Span::new(self.file(), start, end)),
            )));
        }
        Ok(self.target.values.push(ValueNode::new(
            Value::Call {
                name: phrase.to_string(),
                args: Vec::new(),
            },
            Some(Span::new(self.file(), start, end)),
        )))
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
            let canonical_arg_index = if matches!(
                call_id,
                "setPlayerVariableAtIndex" | "modifyPlayerVariableAtIndex"
            ) && arg_index >= 2
                && args.len() == arg_index - 1
            {
                arg_index - 1
            } else {
                arg_index
            };
            // Indexed variable actions carry a project-local variable name,
            // not a Workshop value expression. Resolve it through the symbol
            // table so names such as `Brigitte` remain variable identities.
            if call_id == "string" && arg_index == 0 {
                args.push(self.localized_string_argument()?);
            } else if matches!(
                call_id,
                "setGlobalVariableAtIndex" | "modifyGlobalVariableAtIndex"
            ) && arg_index == 0
            {
                let (name, start, end) = self.phrase()?;
                let variable = self.global_by_name(&name)?;
                args.push(self.target.values.push(ValueNode::new(
                    Value::GlobalVariable(variable),
                    Some(Span::new(self.file(), start, end)),
                )));
            } else if matches!(
                call_id,
                "setPlayerVariableAtIndex" | "modifyPlayerVariableAtIndex"
            ) && arg_index == 1
            {
                let saved = self.pos;
                let (name, start, end) = self.phrase()?;
                if let Some(variable) = self.players.get(&name).copied() {
                    let player = args[0];
                    args[0] = self.target.values.push(ValueNode::new(
                        Value::PlayerVariable { player, variable },
                        Some(Span::new(self.file(), start, end)),
                    ));
                } else {
                    self.pos = saved;
                    let saved_domain = self.expected_domain;
                    self.expected_domain =
                        self.context.expected_domain(call_id, canonical_arg_index);
                    let arg = self.value();
                    self.expected_domain = saved_domain;
                    args.push(self.normalize_contextual_argument(
                        call_id,
                        canonical_arg_index,
                        arg?,
                    ));
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
                // Each argument is parsed with the domain its position expects
                // per the enclosing call's canonical signature (#111); nested
                // calls override the expectation for their own arguments.
                let saved = self.expected_domain;
                self.expected_domain = self
                    .context
                    .expected_domain(call_id, canonical_arg_index)
                    .or_else(|| {
                        (matches!(call_id, "array" | "randomValueInArray"))
                            .then_some(saved)
                            .flatten()
                    });
                let arg = self.value();
                self.expected_domain = saved;
                let arg = self.normalize_contextual_argument(call_id, canonical_arg_index, arg?);
                args.push(self.normalize_modify_call_value(
                    call_id,
                    canonical_arg_index,
                    &args,
                    arg,
                ));
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
                        }) => self.pos += 1,
                        Some(Token {
                            kind: TokenKind::RParen,
                            ..
                        }) => break,
                        Some(token) => return Err(self.malformed("expected ',' or ')'", &token)),
                        None => {
                            return Err(self.malformed("unexpected end of value call", self.eof()));
                        }
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
            || matches!(
                self.canonical_keyword(&phrase).as_str(),
                "True" | "False" | "真" | "假"
            )
            || self
                .peek()
                .is_some_and(|token| matches!(&token.kind, TokenKind::LParen | TokenKind::Dot));
        self.pos = saved;
        if expression_like || !matches!(token.kind, TokenKind::Word(_)) {
            return self.value();
        }

        let (text, start, end) = self.phrase()?;
        self.localized_string_literal(text, Some(Span::new(self.file(), start, end)))
    }

    pub(crate) fn localized_string_literal(
        &mut self,
        text: String,
        span: Option<Span>,
    ) -> Result<wir::ValueId> {
        let Some(entry) = self.catalog.resolve_localized_string(&self.locale, &text) else {
            return Err(WorkshopError::Unknown {
                kind: "localized string",
                spelling: text,
                locale: self.locale.clone(),
                span,
            });
        };
        Ok(self.target.values.push(ValueNode::new(
            Value::LocalizedString(entry.id.clone()),
            span,
        )))
    }

    pub(crate) fn opaque_value_args(&mut self) -> Result<Vec<wir::ValueId>> {
        let mut args = Vec::new();
        if matches!(self.peek().map(|token| token.kind), Some(TokenKind::RParen)) {
            return Ok(args);
        }
        loop {
            args.push(self.value()?);
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Colon,
                    ..
                }) => {
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
                        }) => self.pos += 1,
                        Some(Token {
                            kind: TokenKind::RParen,
                            ..
                        }) => break,
                        Some(token) => return Err(self.malformed("expected ',' or ')'", &token)),
                        None => {
                            return Err(self.malformed("unexpected end of value call", self.eof()));
                        }
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
        self.target.values.push(ValueNode::new(
            Value::Bool(value),
            Some(Span::new(self.file(), start, end)),
        ))
    }
}
