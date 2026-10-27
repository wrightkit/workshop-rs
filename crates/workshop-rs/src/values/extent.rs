//! Authored value extents: the span from a sub-expression's first token
//! through its last token.

use crate::frontend::parser::*;

impl ParseContext<'_> {
    /// End position of the last consumed token when it closes the authored
    /// expression — `)` or `]`. Otherwise the value's own recorded extent is
    /// used, so comments and parenthesized tails never widen an extent past
    /// authored tokens.
    fn extent_end(&self, id: wir::ValueId, end: Position) -> Position {
        match self.previous() {
            Token {
                kind: TokenKind::RParen | TokenKind::RBracket,
                ..
            } => end,
            _ => self
                .target
                .values
                .get(id)
                .and_then(|node| node.extent)
                .map(|extent| extent.end)
                .unwrap_or(end),
        }
    }

    /// Record the authored extent of the value `id` heads: from `start`, the
    /// value's first token, through the end of the expression's last token.
    /// Without a closing `)` or `]` the extent stays inside authored tokens,
    /// never covering trailing comments, `;`, or a following line.
    pub(super) fn with_extent(&mut self, id: wir::ValueId, start: Position) -> wir::ValueId {
        let end = self.extent_end(id, self.previous_span().1);
        let file = self.file();
        if let Some(node) = self.target.values.get_mut(id) {
            node.extent = Some(Span::new(file, start, end));
        }
        id
    }

    /// Push a value node and record its authored extent when the parsed
    /// expression's last token extends past the node's own span.
    pub(super) fn push(&mut self, node: ValueNode, start: Position) -> wir::ValueId {
        let id = self.target.values.push(node);
        self.with_extent(id, start)
    }

    /// Widen an authored extent already recorded on `id`. Child extents
    /// recorded mid-parse never widen past their own authored tokens; once
    /// the enclosing expression closes, the last recorded extent is the full
    /// authored text and stays the boundary for later widening — an update
    /// that would narrow the recorded extent is ignored.
    pub(super) fn extend_extent(
        &mut self,
        id: wir::ValueId,
        start: Position,
        end: Position,
    ) -> wir::ValueId {
        let file = self.file();
        if let Some(node) = self.target.values.get_mut(id) {
            let narrowed = node.extent.is_some_and(|extent| {
                extent.file == file
                    && (start.line, start.col) >= (extent.start.line, extent.start.col)
                    && (end.line, end.col) <= (extent.end.line, extent.end.col)
            });
            if !narrowed {
                node.extent = Some(Span::new(file, start, end));
            }
        }
        id
    }

    /// Push a value node carrying an explicit authored extent.
    pub(super) fn push_extended(&mut self, node: ValueNode, extent: Span) -> wir::ValueId {
        self.target.values.push(node.with_extent(Some(extent)))
    }

    /// The authored extent recorded on a value node, or its node span.
    pub(super) fn extent_or_span(&self, id: wir::ValueId) -> Option<Span> {
        self.target
            .values
            .get(id)
            .and_then(|node| node.extent.or(node.span))
    }

    /// Push the `valueInArray(receiver, index)` node for a postfix
    /// `receiver[index]` after `]` is consumed, recording an extent from
    /// the receiver's first token through the `]`.
    pub(super) fn index_into(
        &mut self,
        receiver: wir::ValueId,
        index: wir::ValueId,
        bracket_start: Position,
    ) -> wir::ValueId {
        let receiver_extent = self.extent_or_span(receiver);
        let end = self.previous_span().1;
        let node = ValueNode::new(
            Value::Call {
                name: "valueInArray".to_string(),
                args: vec![receiver, index],
            },
            Some(Span::new(self.file(), bracket_start, end)),
        );
        match receiver_extent {
            Some(extent) => self.push_extended(node, Span::new(self.file(), extent.start, end)),
            None => self.target.values.push(node),
        }
    }

    pub(crate) fn primary(&mut self) -> Result<wir::ValueId> {
        match self.peek() {
            Some(Token {
                kind: TokenKind::Op(op),
                start,
                ..
            }) if op == "not" => {
                self.pos += 1;
                let saved_condition = std::mem::replace(&mut self.in_condition, false);
                let value = self.primary();
                self.in_condition = saved_condition;
                let value = value?;
                Ok(self.push(
                    ValueNode::new(
                        Value::Call {
                            name: "not".to_string(),
                            args: vec![value],
                        },
                        Some(Span::new(self.file(), start, self.previous_span().1)),
                    ),
                    start,
                ))
            }
            Some(Token {
                kind: TokenKind::Number { .. },
                ..
            }) => {
                // A number may open a declared catalog spelling (`0. Yuva`,
                // `1. Takım`, #448); only a recognized spelling is consumed.
                if let Some((phrase, start, end)) = self.catalog_value_spelling() {
                    return self.catalog_value_call_or_member(phrase, start, end);
                }
                let Some(Token {
                    kind: TokenKind::Number { value, text },
                    start,
                    end,
                }) = self.peek()
                else {
                    unreachable!("number arm entered on a non-number token")
                };
                let span = Some(Span::new(self.file(), start, end));
                self.pos += 1;
                Ok(self.push(ValueNode::new(Value::Number { value, text }, span), start))
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
                    Ok(self.push(
                        ValueNode::new(
                            Value::Number {
                                value: -value,
                                text: format!("-{text}"),
                            },
                            span,
                        ),
                        start,
                    ))
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
                Ok(self.push(ValueNode::new(Value::String(content), span), start))
            }
            Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) if matches!(word.as_str(), "Global" | "global")
                || self.canonical_keyword(&word) == "global" =>
            {
                let (start, end) = self.span_here();
                self.pos += 1;
                // The reference's value spelling `Global Variable(name)`
                // (#119 differential evidence); the OPY form `Global.name`
                // stays supported.
                if let Some(Token {
                    kind: TokenKind::Word(next),
                    ..
                }) = self.peek()
                    && (canonical_keyword(&next) == "Variable"
                        || canonical_keyword(&next) == "variable")
                {
                    self.pos += 1;
                    self.expect(TokenKind::LParen, "expected '(' after 'Global Variable'")?;
                    let (name, name_start, name_end) = self.phrase()?;
                    let variable = self.global_by_name(&name)?;
                    self.expect(TokenKind::RParen, "expected ')' after Global Variable")?;
                    let span = Some(Span::new(self.file(), start, end));
                    return Ok(self.push(
                        ValueNode::new(Value::GlobalVariable(variable), span)
                            .with_identifier(Some(Span::new(self.file(), name_start, name_end))),
                        start,
                    ));
                }
                self.expect(TokenKind::Dot, "expected '.' after 'Global'")?;
                let (name, name_start, name_end) = self.phrase()?;
                let variable = self.global_by_name(&name)?;
                let span = Some(Span::new(self.file(), start, end));
                Ok(self.push(
                    ValueNode::new(Value::GlobalVariable(variable), span)
                        .with_identifier(Some(Span::new(self.file(), name_start, name_end))),
                    start,
                ))
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
                self.with_domain(None, |p| {
                    p.consume_phrase("Variable")?;
                    p.expect(TokenKind::LParen, "expected '(' after 'Player Variable'")?;
                    let player = p.value()?;
                    p.expect(TokenKind::Comma, "expected ',' after player")?;
                    let (name, name_start, name_end) = p.phrase()?;
                    let variable = p.player_by_name(&name)?;
                    p.expect(TokenKind::RParen, "expected ')' after Player Variable")?;
                    let id = p.target.values.push(
                        ValueNode::new(
                            Value::PlayerVariable { player, variable },
                            Some(Span::new(p.file(), start, end)),
                        )
                        .with_identifier(Some(Span::new(
                            p.file(),
                            name_start,
                            name_end,
                        ))),
                    );
                    Ok(p.with_extent(id, start))
                })
            }
            Some(Token {
                kind: TokenKind::Word(word),
                ..
            }) if matches!(canonical_keyword(&word), "Event" | "event" | "Event Player") => {
                let (start, _) = self.span_here();
                self.pos += 1;
                if canonical_keyword(&word) == "Event Player" {
                    return self.event_player_value(start);
                }
                if matches!(self.peek(), Some(Token { kind: TokenKind::Word(next), .. }) if matches!(canonical_keyword(&next), "Player" | "player"))
                {
                    self.pos += 1;
                    return self.event_player_value(start);
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
                // receiver: `(Event Player).p` (#87). The postfix loop in
                // `value()` has already consumed any `.member` inside the
                // parens, so only the post-`)` member access remains.
                // The inner node keeps its full extent: a parenthesized
                // operand's authored text includes its enclosing parens,
                // so extents computed from operand extents bound the whole
                // authored sub-expression.
                let (start, _) = self.span_here();
                self.pos += 1;
                let inner = self.value()?;
                self.expect(TokenKind::RParen, "expected ')' after parenthesized value")?;
                self.extend_extent(inner, start, self.previous_span().1);
                if let Some(Token {
                    kind: TokenKind::Dot,
                    ..
                }) = self.peek()
                {
                    self.pos += 1;
                    let (name, name_start, name_end) = self.phrase()?;
                    if matches!(
                        self.target.values.get(inner),
                        Some(ValueNode {
                            value: Value::EventPlayer,
                            ..
                        })
                    ) || self.players.contains_key(&name)
                    {
                        let variable = self.player_by_name(&name)?;
                        let name_span = Some(Span::new(self.file(), name_start, name_end));
                        Ok(self.push_extended(
                            ValueNode::new(
                                Value::PlayerVariable {
                                    player: inner,
                                    variable,
                                },
                                None,
                            )
                            .with_identifier(name_span),
                            Span::new(self.file(), start, name_end),
                        ))
                    } else {
                        self.member_access(inner, name, name_start, name_end)
                    }
                } else {
                    Ok(inner)
                }
            }
            _ => {
                if let Some((phrase, start, end)) = self.catalog_value_spelling() {
                    return self.catalog_value_call_or_member(phrase, start, end);
                }
                let (phrase, start, end) = self.phrase()?;
                let canonical_value = self
                    .resolve_entry(Kind::Value, &phrase)
                    .map(|entry| entry.id.as_str())
                    .unwrap_or_else(|| canonical_keyword(&phrase));
                match canonical_value {
                    "true" => Ok(self.push_bool(true, start, end)),
                    "false" => Ok(self.push_bool(false, start, end)),
                    "eventPlayer" | "Event Player" => Ok(self.target.values.push(ValueNode::new(
                        Value::EventPlayer,
                        Some(Span::new(self.file(), start, end)),
                    ))),
                    "null" => Ok(self.target.values.push(ValueNode::new(
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
}
