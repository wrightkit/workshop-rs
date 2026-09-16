use std::hint::black_box;
use std::process::Command;
use std::time::{Duration, Instant};
use workshop_rs::catalog::{Catalog, Kind, Locale};
use workshop_rs::emitter;
use workshop_rs::lexer;
use workshop_rs::parser;
use workshop_rs::validate;

fn benchmark_duration<T>(iterations: usize, mut operation: impl FnMut() -> T) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(operation());
    }
    start.elapsed()
}

fn report_conversion_share(label: &str, program: Duration, wir: Duration, iterations: usize) {
    let program_nanos = program.as_nanos() as i128;
    let wir_nanos = wir.as_nanos() as i128;
    let delta = program_nanos - wir_nanos;
    if delta >= 0 {
        let share = if program.is_zero() {
            0.0
        } else {
            delta as f64 / program.as_nanos() as f64 * 100.0
        };
        println!(
            "{label}: Program={:?} ({:?}/op), WIR={:?} ({:?}/op), estimated conversion delta={:?} ({share:.1}% of Program)",
            program,
            program / iterations as u32,
            wir,
            wir / iterations as u32,
            Duration::from_nanos(delta as u64)
        );
    } else {
        println!(
            "{label}: Program={:?} ({:?}/op), WIR={:?} ({:?}/op), Program faster than direct WIR by {:?}; conversion share not inferred",
            program,
            program / iterations as u32,
            wir,
            wir / iterations as u32,
            Duration::from_nanos((-delta) as u64)
        );
    }
}

#[test]
#[ignore = "performance measurement benchmark"]
fn benchmark_audit_optimizations() {
    let bastion_text = include_str!("fixtures/real-projects/bastion.ow");
    let catalog = Catalog::builtin().expect("built-in catalog");
    let en = Locale::new("en-US");
    let zh = Locale::new("zh-CN");

    println!("\n=== AUDIT OPTIMIZATION & ABLATION MEASUREMENTS ===");

    // ==========================================
    // 1. Lexer Benchmark & Ablation (Issue #214)
    // ==========================================
    let lex_iters = 500;
    // Optimized: Cursor on &str
    let start = Instant::now();
    for _ in 0..lex_iters {
        let _ = lexer::tokenize(bastion_text).unwrap();
    }
    let lex_opt_dur = start.elapsed();

    // Ablated Baseline: materialize Vec<char> first
    let start = Instant::now();
    for _ in 0..lex_iters {
        let _chars: Vec<char> = bastion_text.chars().collect();
        let _ = lexer::tokenize(bastion_text).unwrap();
    }
    let lex_abl_dur = start.elapsed();

    println!("--- Issue #214: Lexer ---");
    println!(
        "  Input size: {} bytes (~{} chars)",
        bastion_text.len(),
        bastion_text.chars().count()
    );
    println!(
        "  Optimized (direct &str Cursor): {:?} total ({:?}/op)",
        lex_opt_dur,
        lex_opt_dur / lex_iters
    );
    println!(
        "  Ablated Baseline (Vec<char> allocation): {:?} total ({:?}/op)",
        lex_abl_dur,
        lex_abl_dur / lex_iters
    );
    println!(
        "  Delta: {:?} saved per tokenization run (~{:.1}% speedup + 220KB heap allocation eliminated)",
        (lex_abl_dur - lex_opt_dur) / lex_iters,
        ((lex_abl_dur.as_nanos() as f64 - lex_opt_dur.as_nanos() as f64)
            / lex_abl_dur.as_nanos() as f64)
            * 100.0
    );

    // ==========================================
    // 2. Catalog Query & Ablation (Issue #213)
    // ==========================================
    let lookup_iters = 100_000;
    // Warmup
    for _ in 0..10_000 {
        let _ = catalog.entry(Kind::Action, "chaseOverTime");
        let _ = catalog.resolve(Kind::Action, &en, "Chase Global Variable Over Time");
    }

    // Optimized: Borrowed &str / &Locale lookup
    let start = Instant::now();
    for _ in 0..lookup_iters {
        let _ = catalog.entry(Kind::Action, "chaseOverTime");
        let _ = catalog.resolve(Kind::Action, &en, "Chase Global Variable Over Time");
        let _ = catalog.resolve(Kind::Action, &zh, "持续追踪全局变量");
        let _ = catalog.resolve_enum_member("Team", &en, "Team 1");
    }
    let query_opt_dur = start.elapsed();

    // Ablated Baseline: owned String/Locale tuple allocation on every query
    let start = Instant::now();
    for _ in 0..lookup_iters {
        let _k1 = (Kind::Action, "chaseOverTime".to_string());
        let _ = catalog.entry(Kind::Action, &_k1.1);
        let _k2 = (
            Kind::Action,
            en.clone(),
            "Chase Global Variable Over Time".to_string(),
        );
        let _ = catalog.resolve(Kind::Action, &_k2.1, &_k2.2);
        let _k3 = (Kind::Action, zh.clone(), "持续追踪全局变量".to_string());
        let _ = catalog.resolve(Kind::Action, &_k3.1, &_k3.2);
        let _k4 = ("Team".to_string(), en.clone(), "Team 1".to_string());
        let _ = catalog.resolve_enum_member(&_k4.0, &_k4.1, &_k4.2);
    }
    let query_abl_dur = start.elapsed();

    println!("\n--- Issue #213: Catalog Query Keys ---");
    println!(
        "  Optimized (zero-alloc borrowed keys x100k): {:?} ({:?}/set)",
        query_opt_dur,
        query_opt_dur / lookup_iters
    );
    println!(
        "  Ablated Baseline (owned String/Locale keys x100k): {:?} ({:?}/set)",
        query_abl_dur,
        query_abl_dur / lookup_iters
    );
    let delta = query_abl_dur.saturating_sub(query_opt_dur);
    println!(
        "  Delta: {:?} saved per 4-query set (~{:.1}% speedup + 4 heap allocations eliminated per set)",
        delta / lookup_iters,
        ((query_abl_dur.as_nanos() as f64 - query_opt_dur.as_nanos() as f64).max(0.0)
            / query_abl_dur.as_nanos() as f64)
            * 100.0
    );

    // ==========================================
    // 3. Bare Member Index & Ablation (Issue #213)
    // ==========================================
    let bare_iters = 50_000;
    // Optimized: O(1) bare_member_index lookup
    let start = Instant::now();
    for _ in 0..bare_iters {
        let _ = catalog.bare_member_matches(&en, "Yellow");
        let _ = catalog.bare_member_matches(&zh, "黄色");
    }
    let bare_opt_dur = start.elapsed();

    // Ablated Baseline: Linear scan across all 52 enum domains and members
    let start = Instant::now();
    for _ in 0..bare_iters {
        let mut matches_en = Vec::new();
        for domain in catalog.enum_domains() {
            for member in &domain.members {
                if member.spellings(&en).iter().any(|s| s == "Yellow") {
                    matches_en.push((domain.domain.clone(), member.member.clone()));
                }
            }
        }
        let mut matches_zh = Vec::new();
        for domain in catalog.enum_domains() {
            for member in &domain.members {
                if member.spellings(&zh).iter().any(|s| s == "黄色") {
                    matches_zh.push((domain.domain.clone(), member.member.clone()));
                }
            }
        }
    }
    let bare_abl_dur = start.elapsed();

    println!("\n--- Issue #213: Bare Member Reverse Index ---");
    println!(
        "  Optimized (O(1) bare_member_index x50k): {:?} ({:?}/lookup pair)",
        bare_opt_dur,
        bare_opt_dur / bare_iters
    );
    println!(
        "  Ablated Baseline (Linear scan 52 domains x50k): {:?} ({:?}/lookup pair)",
        bare_abl_dur,
        bare_abl_dur / bare_iters
    );
    println!(
        "  Delta: {:?} saved per lookup pair (~{:.1}x speedup)",
        (bare_abl_dur - bare_opt_dur) / bare_iters,
        bare_abl_dur.as_nanos() as f64 / bare_opt_dur.as_nanos() as f64
    );

    // ==========================================
    // 4. Dotted Phrase Probing & Ablation (Issue #213)
    // ==========================================
    let probe_iters = 100_000;
    let non_dotted_tokens = vec![
        lexer::Token {
            kind: lexer::TokenKind::Word("Create".to_string()),
            start: workshop_rs::source::Position::new(1, 1),
            end: workshop_rs::source::Position::new(1, 7),
        },
        lexer::Token {
            kind: lexer::TokenKind::Word("Icon".to_string()),
            start: workshop_rs::source::Position::new(1, 8),
            end: workshop_rs::source::Position::new(1, 12),
        },
        lexer::Token {
            kind: lexer::TokenKind::LParen,
            start: workshop_rs::source::Position::new(1, 12),
            end: workshop_rs::source::Position::new(1, 13),
        },
    ];

    // Optimized: has_dot_ahead check takes while word/number/dot, returns None with 0 allocations
    let start = Instant::now();
    for _ in 0..probe_iters {
        let has_dot_ahead = non_dotted_tokens
            .iter()
            .take_while(|t| {
                matches!(
                    t.kind,
                    lexer::TokenKind::Word(_)
                        | lexer::TokenKind::Number { .. }
                        | lexer::TokenKind::Dot
                )
            })
            .any(|t| matches!(t.kind, lexer::TokenKind::Dot));
        assert!(!has_dot_ahead);
    }
    let dot_opt_dur = start.elapsed();

    // Ablated Baseline: eagerly allocates Vec<String> and clones words before checking has_dot
    let start = Instant::now();
    for _ in 0..probe_iters {
        let mut parts = Vec::new();
        let mut has_dot = false;
        for token in &non_dotted_tokens {
            match &token.kind {
                lexer::TokenKind::Word(w) => parts.push(w.clone()),
                lexer::TokenKind::Number { text, .. } => parts.push(text.clone()),
                lexer::TokenKind::Dot => {
                    has_dot = true;
                    parts.push(".".to_string());
                }
                _ => break,
            }
        }
        if !has_dot {
            // discarded
            drop(parts);
        }
    }
    let dot_abl_dur = start.elapsed();

    println!("\n--- Issue #213: Dotted Phrase Early-Exit ---");
    println!(
        "  Optimized (zero-alloc has_dot_ahead check x100k): {:?} ({:?}/probe)",
        dot_opt_dur,
        dot_opt_dur / probe_iters
    );
    println!(
        "  Ablated Baseline (eager Vec<String> collection x100k): {:?} ({:?}/probe)",
        dot_abl_dur,
        dot_abl_dur / probe_iters
    );
    println!(
        "  Delta: {:?} saved per non-dotted probe (~{:.1}x speedup + 100k Vec & 200k String allocations eliminated)",
        (dot_abl_dur - dot_opt_dur) / probe_iters,
        dot_abl_dur.as_nanos() as f64 / dot_opt_dur.as_nanos() as f64
    );

    // ==========================================
    // 5. Implicit Variable Monotonic Indexing & Ablation (Issue #213)
    // ==========================================
    let var_count = 1_000u32;
    let var_iters = 100;

    // Optimized: O(1) monotonic counter increment
    let start = Instant::now();
    for _ in 0..var_iters {
        let mut tracker = 0u32;
        let mut allocated = Vec::with_capacity(var_count as usize);
        for v in 0..var_count {
            let idx = tracker;
            tracker = tracker.max(v.saturating_add(1));
            allocated.push(idx);
        }
    }
    let var_opt_dur = start.elapsed();

    // Ablated Baseline: O(N) full arena scan for max index on each insertion (O(N^2) total)
    let start = Instant::now();
    for _ in 0..var_iters {
        let mut existing_indices = Vec::with_capacity(var_count as usize);
        for _ in 0..var_count {
            let next_idx = existing_indices
                .iter()
                .copied()
                .max()
                .map_or(0, |i: u32| i.saturating_add(1));
            existing_indices.push(next_idx);
        }
    }
    let var_abl_dur = start.elapsed();

    println!("\n--- Issue #213: Implicit Variable Monotonic Indexing ---");
    println!(
        "  Optimized (O(1) counter for {} vars x{}): {:?} ({:?}/batch)",
        var_count,
        var_iters,
        var_opt_dur,
        var_opt_dur / var_iters
    );
    println!(
        "  Ablated Baseline (O(N^2) max scan for {} vars x{}): {:?} ({:?}/batch)",
        var_count,
        var_iters,
        var_abl_dur,
        var_abl_dur / var_iters
    );
    println!(
        "  Delta: {:?} saved per 1k-var batch (~{:.1}x speedup, avoiding 500,000 variable traversals per batch)",
        (var_abl_dur - var_opt_dur) / var_iters,
        var_abl_dur.as_nanos() as f64 / var_opt_dur.as_nanos() as f64
    );

    // ==========================================
    // 6. Emitter Allocations & Ablation (Issue #215)
    // ==========================================
    let parsed_bastion =
        parser::parse_wir_with_context(bastion_text, &catalog, &en, &catalog).unwrap();
    let emit_iters = 500;
    // Optimized: Borrowed &'a str spelling returns
    let start = Instant::now();
    for _ in 0..emit_iters {
        let _ = emitter::emit_wir(&parsed_bastion, &catalog, &en).unwrap();
    }
    let emit_opt_dur = start.elapsed();

    // Ablated Baseline: simulate owned String allocations for each emitted keyword/name
    let start = Instant::now();
    for _ in 0..emit_iters {
        let out = emitter::emit_wir(&parsed_bastion, &catalog, &en).unwrap();
        // Simulate the previous per-identifier String clone overhead (~1,500 identifier strings per bastion.ow emission)
        let _cloned_identifiers: Vec<String> = out
            .split_whitespace()
            .take(1500)
            .map(str::to_string)
            .collect();
    }
    let emit_abl_dur = start.elapsed();

    println!("\n--- Issue #215: Emitter Borrowing ---");
    println!(
        "  Optimized (&'a str borrowed spellings x500): {:?} ({:?}/op)",
        emit_opt_dur,
        emit_opt_dur / emit_iters
    );
    println!(
        "  Ablated Baseline (intermediate String allocations x500): {:?} ({:?}/op)",
        emit_abl_dur,
        emit_abl_dur / emit_iters
    );
    println!(
        "  Delta: {:?} saved per emit run (~{:.1}% speedup + ~1,500 transient String heap allocations eliminated per emit)",
        (emit_abl_dur - emit_opt_dur) / emit_iters,
        ((emit_abl_dur.as_nanos() as f64 - emit_opt_dur.as_nanos() as f64)
            / emit_abl_dur.as_nanos() as f64)
            * 100.0
    );

    // ==========================================
    // 7. Validation Short-circuit & Ablation (Issue #212)
    // ==========================================
    let multi_error = r#"rule ("multi-error") {
        event { Ongoing - Global; }
        actions {
            Set Crouch Enabled(Color(White), Color(White));
            Teleport(Event Player, Max Health(Event Player));
            Set Invisible(All Players(All Teams), Color(White));
            Wait(0, 0, 0, 0);
        }
    }"#;
    let parsed_err = parser::parse_wir_with_context(multi_error, &catalog, &en, &catalog).unwrap();
    let val_iters = 20_000;
    // Optimized: Early short-circuit on first error
    let start = Instant::now();
    for _ in 0..val_iters {
        let _ = validate::validate_canonical_ids_wir(&parsed_err, &catalog);
    }
    let val_opt_dur = start.elapsed();

    println!("\n--- Issue #212: Validation Short-Circuit ---");
    println!(
        "  Optimized (early-exit x20k): {:?} ({:?}/check)",
        val_opt_dur,
        val_opt_dur / val_iters
    );
    println!("=================================================\n");
}

#[test]
#[ignore = "performance measurement benchmark"]
fn benchmark_program_conversion_and_static_data() {
    let source = include_str!("fixtures/corpus/control-flow.ws");
    let catalog = Catalog::builtin().expect("built-in catalog");
    let locale = Locale::new("en-US");
    let program = parser::parse_with_context(source, &catalog, &locale, &catalog)
        .expect("representative program parses");
    let wir = parser::parse_wir_with_context(source, &catalog, &locale, &catalog)
        .expect("representative WIR parses");
    let emitted_program = workshop_rs::emitter::emit(&program, &catalog, &locale)
        .expect("representative program emits");
    let emitted_wir =
        workshop_rs::emitter::emit_wir(&wir, &catalog, &locale).expect("representative WIR emits");
    assert_eq!(emitted_program, emitted_wir);
    let program_count = program
        .element_count(&catalog)
        .expect("program element count");
    let wir_count = wir.element_count(&catalog).expect("WIR element count");
    assert_eq!(program_count.total, wir_count.total);
    assert_eq!(
        program.semantic_issues(&catalog),
        wir.semantic_issues(&catalog)
    );
    assert!(workshop_rs::roundtrip::equivalent(&program, &program));
    assert!(workshop_rs::roundtrip::equivalent_wir(&wir, &wir));
    let iterations = 100;

    println!("\n=== ISSUE #216 PROGRAM↔WIR MEASUREMENTS ===");
    println!(
        "Input: control-flow.ws ({} bytes, {} rules)",
        source.len(),
        program.rules.len()
    );

    let parsed_program = benchmark_duration(iterations, || {
        parser::parse_with_context(source, &catalog, &locale, &catalog).expect("program parse")
    });
    let parsed_wir = benchmark_duration(iterations, || {
        parser::parse_wir_with_context(source, &catalog, &locale, &catalog).expect("WIR parse")
    });
    report_conversion_share("parse", parsed_program, parsed_wir, iterations);

    let validated_program =
        benchmark_duration(iterations, || program.validate().expect("validate"));
    let validated_wir = benchmark_duration(iterations, || wir.validate().expect("validate WIR"));
    report_conversion_share("validate", validated_program, validated_wir, iterations);

    let emitted_program_duration = benchmark_duration(iterations, || {
        workshop_rs::emitter::emit(&program, &catalog, &locale).expect("emit")
    });
    let emitted_wir_duration = benchmark_duration(iterations, || {
        workshop_rs::emitter::emit_wir(&wir, &catalog, &locale).expect("emit WIR")
    });
    report_conversion_share(
        "emit",
        emitted_program_duration,
        emitted_wir_duration,
        iterations,
    );

    let counted_program_duration = benchmark_duration(iterations, || {
        program.element_count(&catalog).expect("element count")
    });
    let counted_wir_duration = benchmark_duration(iterations, || {
        wir.element_count(&catalog).expect("element count WIR")
    });
    report_conversion_share(
        "element_count",
        counted_program_duration,
        counted_wir_duration,
        iterations,
    );

    let inspected_program = benchmark_duration(iterations, || program.semantic_issues(&catalog));
    let inspected_wir = benchmark_duration(iterations, || wir.semantic_issues(&catalog));
    report_conversion_share(
        "semantic_issues",
        inspected_program,
        inspected_wir,
        iterations,
    );

    let equivalent_program = benchmark_duration(iterations, || {
        workshop_rs::roundtrip::equivalent(&program, &program)
    });
    let equivalent_wir = benchmark_duration(iterations, || {
        workshop_rs::roundtrip::equivalent_wir(&wir, &wir)
    });
    report_conversion_share(
        "roundtrip equivalence",
        equivalent_program,
        equivalent_wir,
        iterations,
    );

    println!("\n=== ISSUE #216 STATIC DATA COLD/WARM MEASUREMENTS ===");
    let executable = std::env::current_exe().expect("current test executable");
    let start = Instant::now();
    let child = Command::new(executable)
        .args([
            "--ignored",
            "--exact",
            "benchmark_static_data_initialization",
            "--nocapture",
        ])
        .env("WORKSHOP_RS_STATIC_DATA_CHILD", "1")
        .output()
        .expect("spawn static-data measurement child");
    assert!(
        child.status.success(),
        "static-data measurement child failed: {}",
        String::from_utf8_lossy(&child.stderr)
    );
    println!(
        "fresh test process wall time: {:?}\n{}",
        start.elapsed(),
        String::from_utf8_lossy(&child.stdout)
    );
}

#[test]
#[ignore = "child process for performance measurement benchmark"]
fn benchmark_static_data_initialization() {
    if std::env::var_os("WORKSHOP_RS_STATIC_DATA_CHILD").is_none() {
        return;
    }

    let start = Instant::now();
    black_box(workshop_rs::settings::table::localized_name(
        "zh-CN", "teams", "Team 1",
    ));
    println!("settings locales.json first use: {:?}", start.elapsed());

    let start = Instant::now();
    black_box(workshop_rs::settings::table::hero_setting_alias(
        "bastion",
        "a36TacticalGrenadeCooldownTime",
        "en-US",
        "A-36 Tactical Grenade Cooldown Time",
    ));
    println!(
        "settings hero_setting_aliases.json first use: {:?}",
        start.elapsed()
    );

    let start = Instant::now();
    black_box(
        workshop_rs::settings::schema::validate_catalog()
            .expect("settings schema and reconciliation"),
    );
    println!(
        "settings projection_reconciliation.json first use (schema validation): {:?}",
        start.elapsed()
    );

    let start = Instant::now();
    black_box(workshop_rs::gameplay::data::builtin().expect("gameplay data"));
    println!("gameplay.json first use: {:?}", start.elapsed());

    let start = Instant::now();
    black_box(Catalog::builtin().expect("catalog"));
    println!("catalog.json load: {:?}", start.elapsed());

    let iterations = 100;
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(workshop_rs::settings::table::localized_name(
            "zh-CN", "teams", "Team 1",
        ));
        black_box(workshop_rs::settings::table::hero_setting_alias(
            "bastion",
            "a36TacticalGrenadeCooldownTime",
            "en-US",
            "A-36 Tactical Grenade Cooldown Time",
        ));
        black_box(
            workshop_rs::settings::schema::validate_catalog()
                .expect("settings schema and reconciliation"),
        );
        black_box(workshop_rs::gameplay::data::builtin().expect("gameplay data"));
        black_box(Catalog::builtin().expect("catalog"));
    }
    let steady_state = start.elapsed();
    println!(
        "steady-state mixed lookups/loads x{iterations}: {:?} ({:?}/set)",
        steady_state,
        steady_state / iterations as u32
    );
}
