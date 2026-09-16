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

    println!("\n=== REPRESENTATIVE MEASUREMENTS ===");

    // 1. Lexer Benchmark (Issue #214)
    let iters = 500;
    let start = Instant::now();
    for _ in 0..iters {
        let _ = lexer::tokenize(bastion_text).unwrap();
    }
    let lexer_dur = start.elapsed();
    println!(
        "Lexer (Cursor on &str, bastion.ow x{}): {:?} (avg: {:?}/op, len={} bytes)",
        iters,
        lexer_dur,
        lexer_dur / iters,
        bastion_text.len()
    );

    // 2. Catalog Lookup Benchmark (Issue #213)
    let lookup_iters = 50_000;
    let start = Instant::now();
    for _ in 0..lookup_iters {
        let _ = catalog.entry(Kind::Action, "chaseOverTime");
        let _ = catalog.resolve(Kind::Action, &en, "Chase Global Variable Over Time");
        let _ = catalog.resolve(Kind::Action, &zh, "持续追踪全局变量");
        let _ = catalog.resolve_enum_member("Team", &en, "Team 1");
        let _ = catalog.bare_member_matches(&en, "Yellow");
        let _ = catalog.bare_member_matches(&zh, "黄色");
    }
    let catalog_dur = start.elapsed();
    println!(
        "Catalog queries (Zero-allocation indexed lookups x{} sets): {:?} (avg: {:?}/set)",
        lookup_iters,
        catalog_dur,
        catalog_dur / lookup_iters
    );

    // 3. Full Parse Benchmark (Issue #212 & #213)
    let parse_iters = 200;
    let start = Instant::now();
    for _ in 0..parse_iters {
        let _ = parser::parse_wir_with_context(bastion_text, &catalog, &en, &catalog).unwrap();
    }
    let parse_dur = start.elapsed();
    println!(
        "Parser (bastion.ow x{}): {:?} (avg: {:?}/op)",
        parse_iters,
        parse_dur,
        parse_dur / parse_iters
    );

    // 4. Emitter Benchmark (Issue #215)
    let parsed_bastion =
        parser::parse_wir_with_context(bastion_text, &catalog, &en, &catalog).unwrap();
    let emit_iters = 500;
    let start = Instant::now();
    for _ in 0..emit_iters {
        let _ = emitter::emit_wir(&parsed_bastion, &catalog, &en).unwrap();
    }
    let emit_dur = start.elapsed();
    println!(
        "Emitter (Zero-allocation spelling borrow, bastion.ow x{}): {:?} (avg: {:?}/op)",
        emit_iters,
        emit_dur,
        emit_dur / emit_iters
    );

    // 5. Validation Short-circuit Benchmark (Issue #212)
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
    let val_iters = 10_000;
    let start = Instant::now();
    for _ in 0..val_iters {
        let _ = validate::validate_canonical_ids_wir(&parsed_err, &catalog);
    }
    let val_dur = start.elapsed();
    println!(
        "Validation (Early-exit short-circuit x{}): {:?} (avg: {:?}/op)",
        val_iters,
        val_dur,
        val_dur / val_iters
    );
    println!("====================================\n");
}
