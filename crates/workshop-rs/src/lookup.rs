//! Canonical Workshop name lookup: display names, near spellings, and
//! settings paths resolve to the catalog entries, enum members, and settings
//! definitions the parser and emitter accept.
//!
//! [`Catalog::lookup`] answers in canonical Workshop terms: which identity a
//! display name or guess refers to, which parameters and argument domains an
//! action or value declares, which members an enum domain accepts, and which
//! settings keys, kinds, and value forms a path or display name selects.
//! Every answer derives from the same catalog and settings table that drive
//! parsing and emission, so lookup results cannot drift from the accepted
//! vocabulary. Consumers compose and present the matches; this module owns
//! no data of its own.

use crate::catalog::{Catalog, CatalogEntry, Kind, Locale};
use crate::core::error::{Result, WorkshopError};
use crate::core::suggest;
use crate::settings::{self, SettingDefinition};

/// Members of a parameter's enum domain are listed inline in a signature
/// when the domain has at most this many members. Larger domains report
/// their member count; a query on the domain itself lists them.
const INLINE_MEMBER_LIMIT: usize = 32;

/// One canonical construct matching a [`Catalog::lookup`] query.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum LookupMatch {
    /// A catalog builtin: structural keyword, action, value, event, or
    /// operator.
    Builtin {
        /// The entry's catalog kind.
        kind: Kind,
        /// The canonical identity (`createHudText`, `global`).
        id: String,
        /// The display name in the requested locale, when mapped.
        display_name: Option<String>,
        /// The call signature, for actions and values.
        signature: Option<Signature>,
    },
    /// A member of a canonical enum domain (`Team.ALL`, `Hero.SOLDIER_76`).
    EnumMember {
        /// The canonical domain name.
        domain: String,
        /// The canonical member id.
        member: String,
        /// The member's display name in the requested locale, when mapped.
        display_name: Option<String>,
    },
    /// A canonical enum domain and the members it accepts.
    EnumDomain {
        /// The canonical domain name.
        domain: String,
        /// The domain's display name in the requested locale, when mapped.
        display_name: Option<String>,
        /// Every member the domain accepts.
        members: Vec<LookupEnumMember>,
    },
    /// A reviewed settings-table definition: a valid key with its path,
    /// kind, and value forms.
    Setting {
        /// The canonical semantic definition.
        definition: SettingDefinition,
        /// The display name in the requested locale, or the English name
        /// when the locale has no mapping.
        display_name: String,
    },
}

/// A canonical enum member reported by a lookup.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct LookupEnumMember {
    /// The canonical member id (`SOLDIER_76`, `ALL`).
    pub id: String,
    /// The member's display name in the requested locale, when mapped.
    pub display_name: Option<String>,
}

/// An enum domain referenced by a [`Signature`] parameter, with the members
/// it accepts.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SignatureDomain {
    /// The canonical domain name.
    pub domain: String,
    /// Every member the domain accepts.
    pub members: Vec<LookupEnumMember>,
}

/// A compact signature for a catalog action or value.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Signature {
    /// The signature rendered in the requested locale's call syntax:
    /// parameters appear in call order, a parameter without a declared
    /// default reads `name: Type`, a parameter with a declared default reads
    /// `name=default` or `name?` when the default is null, and an enum-typed
    /// parameter without a default reads `name: Domain(members)` listing the
    /// domain's members once per signature when it has at most
    /// 32 members, or `name: Domain(count members)` for a larger domain.
    pub text: String,
    /// Parameters in call order.
    pub params: Vec<SignatureParam>,
    /// The argument count a call must supply: positions below this index
    /// cannot be omitted; declared defaults only make the suffix optional.
    pub required_params: usize,
    /// Whether the final parameter repeats for extra arguments.
    pub variadic: bool,
    /// The source-backed return type for values, when declared.
    pub return_type: Option<String>,
    /// The enum domains referenced by parameters, each with the members it
    /// accepts, for follow-up inspection of large domains.
    pub domains: Vec<SignatureDomain>,
}

/// One parameter of a [`Signature`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SignatureParam {
    /// The semantic parameter name, or the catalog parameter name.
    pub name: String,
    /// The source-backed semantic type, when declared.
    pub param_type: Option<String>,
    /// The canonical enum domain expected at this position, when declared.
    pub domain: Option<String>,
    /// The declared default applied when the argument is omitted. `None`
    /// marks a parameter with no declared default; a declared default before
    /// another required position does not make this position omittable (see
    /// [`Signature::required_params`]).
    pub default: Option<String>,
}

impl Catalog {
    /// Find canonical constructs matching `query` under `locale`.
    ///
    /// `query` may be a display name in `locale`, a primary-locale display
    /// name, a canonical id, a near spelling or guess, or a settings path
    /// prefix (`gamemodes.control`, `heroes.<team>.<hero>`). Matches rank
    /// exact spellings first, then near spellings, token prefixes, and
    /// substring guesses; ties keep catalog and table order. All matches are
    /// canonical constructs the parser and emitter accept, with the display
    /// name in `locale` and, for actions and values, the compact signature.
    ///
    /// An empty result reports that nothing matched. `Err` reports a query
    /// the catalog cannot answer: a locale the catalog does not declare.
    ///
    /// ```
    /// use workshop_rs::catalog::{Catalog, Locale};
    /// use workshop_rs::lookup::LookupMatch;
    ///
    /// let catalog = Catalog::builtin().unwrap();
    /// let matches = catalog.lookup(&Locale::new("en-US"), "create hud txt").unwrap();
    /// assert!(matches.iter().any(|m| matches!(
    ///     m,
    ///     LookupMatch::Builtin { id, .. } if id == "createHudText"
    /// )));
    /// ```
    pub fn lookup(&self, locale: &Locale, query: &str) -> Result<Vec<LookupMatch>> {
        if !self.supports(locale) {
            return Err(WorkshopError::unsupported(
                format!(
                    "lookup for locale '{locale}' is not supported: \
                     the catalog does not declare it"
                ),
                None,
            ));
        }
        let primary = self.primary_locale();
        let distinct_locale = locale != primary;
        let prepared = PreparedQuery::new(query);
        let query_segments: Vec<&str> = query.split('.').collect();
        let mut matches: Vec<(u32, LookupMatch)> = Vec::new();
        for entry in &self.entries {
            let primary_spellings: &[String] = if distinct_locale {
                entry.spellings(primary)
            } else {
                &[]
            };
            let score = std::iter::once(entry.id.as_str())
                .chain(entry.spellings(locale).iter().map(String::as_str))
                .chain(primary_spellings.iter().map(String::as_str))
                .filter_map(|text| prepared.name_score(text))
                .min();
            if let Some(score) = score {
                matches.push((
                    score,
                    LookupMatch::Builtin {
                        kind: entry.kind,
                        id: entry.id.clone(),
                        display_name: entry.spelling(locale).map(String::from),
                        signature: signature(self, entry, locale),
                    },
                ));
            }
        }
        for domain in self.enum_domains() {
            let score = std::iter::once(domain.domain.as_str())
                .chain(domain.spelling(locale))
                .chain(domain.spelling(primary).filter(|_| distinct_locale))
                .filter_map(|text| prepared.name_score(text))
                .min();
            if let Some(score) = score {
                matches.push((
                    score,
                    LookupMatch::EnumDomain {
                        domain: domain.domain.clone(),
                        display_name: domain.spelling(locale).map(String::from),
                        members: self.enum_members(&domain.domain, locale),
                    },
                ));
            }
        }
        for domain in self.enum_domains() {
            for member in &domain.members {
                let qualified = format!("{}.{}", domain.domain, member.member);
                let primary_spellings: &[String] = if distinct_locale {
                    member.spellings(primary)
                } else {
                    &[]
                };
                let score = std::iter::once(member.member.as_str())
                    .chain(std::iter::once(qualified.as_str()))
                    .chain(member.spellings(locale).iter().map(String::as_str))
                    .chain(primary_spellings.iter().map(String::as_str))
                    .filter_map(|text| prepared.name_score(text))
                    .min();
                if let Some(score) = score {
                    matches.push((
                        score,
                        LookupMatch::EnumMember {
                            domain: domain.domain.clone(),
                            member: member.member.clone(),
                            display_name: self
                                .enum_spelling(&domain.domain, locale, &member.member)
                                .map(String::from),
                        },
                    ));
                }
            }
        }
        for definition in settings::definitions() {
            let name_texts = [
                Some(definition.path().rsplit('.').next().unwrap_or_default()),
                Some(definition.presentation().english_name),
                definition.presentation().localized_name(locale.as_str()),
                definition.id().map(|id| id.as_str()),
                Some(definition.path()),
            ];
            let score = name_texts
                .into_iter()
                .flatten()
                .filter_map(|text| prepared.name_score(text))
                .min()
                .into_iter()
                .chain(path_prefix_score(&query_segments, definition.path()))
                .min();
            if let Some(score) = score {
                matches.push((
                    score,
                    LookupMatch::Setting {
                        display_name: definition
                            .presentation()
                            .localized_name(locale.as_str())
                            .unwrap_or(definition.presentation().english_name)
                            .to_string(),
                        definition,
                    },
                ));
            }
        }
        // Stable sort: equal-ranked matches keep catalog and table order.
        matches.sort_by_key(|(score, _)| *score);
        Ok(matches.into_iter().map(|(_, lookup)| lookup).collect())
    }

    /// The members of `domain` with their display names under `locale`.
    fn enum_members(&self, domain: &str, locale: &Locale) -> Vec<LookupEnumMember> {
        self.enum_domain(domain)
            .map(|domain| {
                domain
                    .members
                    .iter()
                    .map(|member| LookupEnumMember {
                        id: member.member.clone(),
                        display_name: self
                            .enum_spelling(&domain.domain, locale, &member.member)
                            .map(String::from),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// The [`Signature`] for a catalog `entry` under `locale`, when the entry is
/// an action or value.
fn signature(catalog: &Catalog, entry: &CatalogEntry, locale: &Locale) -> Option<Signature> {
    if !matches!(entry.kind, Kind::Action | Kind::Value) {
        return None;
    }
    let params: Vec<SignatureParam> = (0..entry.param_count())
        .map(|index| SignatureParam {
            name: entry.param_name(index).unwrap_or_default().to_string(),
            param_type: entry.param_type(index).map(String::from),
            domain: entry.param_domain(index).map(String::from),
            default: entry.param_default(index).map(String::from),
        })
        .collect();
    let mut domains = Vec::new();
    for param in &params {
        let Some(domain) = &param.domain else {
            continue;
        };
        if domains
            .iter()
            .any(|known: &SignatureDomain| known.domain == *domain)
        {
            continue;
        }
        domains.push(SignatureDomain {
            domain: domain.clone(),
            members: catalog.enum_members(domain, locale),
        });
    }
    Some(Signature {
        text: signature_text(catalog, entry, locale, &params),
        params,
        required_params: entry.required_param_count(),
        variadic: entry.is_variadic(),
        return_type: entry.return_type().map(String::from),
        domains,
    })
}

/// Render `entry`'s call signature in `locale`'s call syntax, following the
/// shared signature form: `name: Type` without a declared default,
/// `name=default` or `name?` with one, and `name: Domain(members|count)` for
/// an enum-typed parameter without a default.
fn signature_text(
    catalog: &Catalog,
    entry: &CatalogEntry,
    locale: &Locale,
    params: &[SignatureParam],
) -> String {
    let display = entry.spelling(locale).unwrap_or(entry.id.as_str());
    let mut listed_domains = std::collections::HashSet::new();
    let mut rendered: Vec<String> = Vec::with_capacity(params.len());
    for param in params {
        rendered.push(match &param.default {
            Some(default) if default == "null" => format!("{}?", param.name),
            Some(default) => format!(
                "{}={}",
                param.name,
                render_default(catalog, locale, default)
            ),
            None => match &param.domain {
                Some(domain) => {
                    let Some(domain_entry) = catalog.enum_domain(domain) else {
                        match &param.param_type {
                            Some(param_type) => {
                                rendered.push(format!("{}: {}", param.name, param_type));
                            }
                            None => rendered.push(param.name.clone()),
                        }
                        continue;
                    };
                    let domain_display = domain_entry
                        .spelling(locale)
                        .unwrap_or(domain_entry.domain.as_str());
                    if !listed_domains.insert(domain.clone()) {
                        format!("{}: {}", param.name, domain_display)
                    } else if domain_entry.members.len() <= INLINE_MEMBER_LIMIT {
                        let members = domain_entry
                            .members
                            .iter()
                            .map(|member| {
                                catalog
                                    .enum_spelling(&domain_entry.domain, locale, &member.member)
                                    .unwrap_or(member.member.as_str())
                            })
                            .collect::<Vec<_>>()
                            .join("|");
                        format!("{}: {}({})", param.name, domain_display, members)
                    } else {
                        format!(
                            "{}: {}({} members)",
                            param.name,
                            domain_display,
                            domain_entry.members.len()
                        )
                    }
                }
                None => match &param.param_type {
                    Some(param_type) => format!("{}: {}", param.name, param_type),
                    None => param.name.clone(),
                },
            },
        });
    }
    if entry.is_variadic() {
        rendered.push("...".to_string());
    }
    format!("{}({})", display, rendered.join(", "))
}

/// Render a declared parameter default in `locale`'s call syntax: a
/// `Domain.Member` literal resolves to the localized domain and member
/// spellings, a canonical value id resolves to its localized spelling, and
/// any other literal stays as declared.
fn render_default(catalog: &Catalog, locale: &Locale, default: &str) -> String {
    if let Some((domain, member)) = default.split_once('.') {
        if let Some(member_spelling) = catalog.enum_spelling(domain, locale, member) {
            let domain_display = catalog
                .enum_domain(domain)
                .and_then(|domain| domain.spelling(locale))
                .unwrap_or(domain);
            return format!("{domain_display}.{member_spelling}");
        }
    }
    catalog
        .spelling(Kind::Value, locale, default)
        .unwrap_or(default)
        .to_string()
}

/// A lookup query prepared once for comparison against every candidate:
/// the raw text, its comparison fold, and its folded tokens.
struct PreparedQuery {
    raw: String,
    folded: String,
    folded_chars: Vec<char>,
    tokens: Vec<String>,
    max_distance: usize,
}

impl PreparedQuery {
    fn new(query: &str) -> Self {
        let folded = suggest::fold_loose(query);
        let folded_chars: Vec<char> = folded.chars().collect();
        Self {
            raw: query.to_string(),
            max_distance: suggest::max_distance(folded_chars.len()),
            folded,
            folded_chars,
            tokens: query
                .split(|character: char| !character.is_alphanumeric())
                .map(suggest::fold_loose)
                .filter(|token| !token.is_empty())
                .collect(),
        }
    }

    /// How closely the query resembles `candidate`: exact equality ranks 0,
    /// a comparison-fold equality (case, accents, punctuation, and
    /// whitespace are insignificant) ranks 1, a small edit distance ranks
    /// 2 + distance, a token prefix match ranks 6, and a substring guess
    /// ranks 8. Guesses shorter than three folded characters only rank on
    /// exact or near forms so that vague queries do not flood the result.
    fn name_score(&self, candidate: &str) -> Option<u32> {
        if self.raw == candidate {
            return Some(0);
        }
        let candidate_folded = fold_candidate(candidate);
        if self.folded.is_empty() || candidate_folded.is_empty() {
            return None;
        }
        if self.folded == candidate_folded {
            return Some(1);
        }
        if candidate_folded
            .chars()
            .count()
            .abs_diff(self.folded_chars.len())
            <= self.max_distance
        {
            let candidate_chars: Vec<char> = candidate_folded.chars().collect();
            let distance =
                suggest::edit_distance(&self.folded_chars, &candidate_chars, self.max_distance);
            if distance <= self.max_distance {
                return Some(2 + distance as u32);
            }
        }
        if self.folded_chars.len() >= 3 {
            if self.token_prefix_match(candidate) {
                return Some(6);
            }
            if candidate_folded.contains(&self.folded) {
                return Some(8);
            }
        }
        None
    }

    /// Whether every whitespace/punctuation-separated query token is a
    /// prefix of some `candidate` token (`hud text` matches
    /// `Create HUD Text`, `create hud` matches `Create HUD Text`).
    fn token_prefix_match(&self, candidate: &str) -> bool {
        let candidate_tokens = candidate
            .split(|character: char| !character.is_alphanumeric())
            .map(fold_candidate)
            .filter(|token| !token.is_empty());
        !self.tokens.is_empty()
            && self.tokens.iter().all(|token| {
                candidate_tokens
                    .clone()
                    .any(|candidate| candidate.starts_with(token))
            })
    }
}

/// The [`suggest::fold_loose`] of `candidate`, fast-pathed for ASCII
/// spellings (the overwhelmingly common catalog and table form).
fn fold_candidate(candidate: &str) -> String {
    if candidate.is_ascii() {
        candidate
            .bytes()
            .filter(u8::is_ascii_alphanumeric)
            .map(|byte| char::from(byte.to_ascii_lowercase()))
            .collect()
    } else {
        suggest::fold_loose(candidate)
    }
}

/// Whether `query_segments` prefix-matches `path` segment for segment: a
/// `<team>` or `<hero>` template segment accepts any query segment, and a
/// literal segment accepts a case-insensitive equal query segment. A full
/// match ranks 2; each unmatched trailing path segment lowers the rank.
fn path_prefix_score(query_segments: &[&str], path: &str) -> Option<u32> {
    let mut path_segments = path.split('.');
    let mut consumed = 0usize;
    for segment in query_segments {
        match path_segments.next() {
            Some("<team>") | Some("<hero>") => {}
            Some(part) if part.eq_ignore_ascii_case(segment) => {}
            _ => return None,
        }
        consumed += 1;
    }
    let depth = path.matches('.').count() + 1;
    Some(2 + (depth - consumed) as u32)
}
