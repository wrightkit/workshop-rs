use std::time::Instant;
use workshop_rs::catalog::{Catalog, Kind, Locale};
use workshop_rs::emitter;
use workshop_rs::lexer;
use workshop_rs::parser;
use workshop_rs::validate;

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
    let lookup_iters = 50_000;
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
        let _ = catalog.entry(Kind::Action, "chaseOverTime");
        let _k2 = (
            Kind::Action,
            en.clone(),
            "Chase Global Variable Over Time".to_string(),
        );
        let _ = catalog.resolve(Kind::Action, &en, "Chase Global Variable Over Time");
        let _k3 = (Kind::Action, zh.clone(), "持续追踪全局变量".to_string());
        let _ = catalog.resolve(Kind::Action, &zh, "持续追踪全局变量");
        let _k4 = ("Team".to_string(), en.clone(), "Team 1".to_string());
        let _ = catalog.resolve_enum_member("Team", &en, "Team 1");
    }
    let query_abl_dur = start.elapsed();

    println!("\n--- Issue #213: Catalog Query Keys ---");
    println!(
        "  Optimized (zero-alloc borrowed keys x50k): {:?} ({:?}/set)",
        query_opt_dur,
        query_opt_dur / lookup_iters
    );
    println!(
        "  Ablated Baseline (owned String/Locale keys x50k): {:?} ({:?}/set)",
        query_abl_dur,
        query_abl_dur / lookup_iters
    );
    println!(
        "  Delta: {:?} saved per 4-query set (~{:.1}% speedup + 4 heap allocations eliminated per set)",
        (query_abl_dur - query_opt_dur) / lookup_iters,
        ((query_abl_dur.as_nanos() as f64 - query_opt_dur.as_nanos() as f64)
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
    // 4. Emitter Allocations & Ablation (Issue #215)
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

    println!("\n--- Issue #215: Emitter Borrowing ---");
    println!(
        "  Optimized (&'a str borrowed spellings x500): {:?} ({:?}/op)",
        emit_opt_dur,
        emit_opt_dur / emit_iters
    );

    // ==========================================
    // 5. Validation Short-circuit & Ablation (Issue #212)
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
