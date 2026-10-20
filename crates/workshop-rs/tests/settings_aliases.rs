//! Hero settings producer aliases: the spellings the pinned OverPy producer
//! writes for hero settings where they differ from the export labels, split
//! out of `settings_pipeline.rs` under the size guard (workshop-rs#540).

use super::common::{catalog, en, zh};
use super::internal;
use workshop_rs::roundtrip;

#[test]
fn pinned_ai_hero_setting_aliases_are_canonical() {
    let catalog = catalog();
    let source = "设置 { 英雄 { 综合 { 索杰恩 { 充能速度 Charged Shot: 200% } 路霸 { secondaryFireRechargeRate%: 150 } 骇灾 { 尖刺护体资源恢复倍率: 150% 尖刺护体资源消耗倍率: 50% } } } }";
    let program = internal::parse_in(source, &zh());
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| { issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting })
    );
}

#[test]
fn pinned_hero_setting_names_emit_the_oracle_spellings() {
    let catalog = catalog();
    let en = en();
    let source = "settings { heroes { General { \
        Ana { No Automatic Fire: Off No Scope: Off } \
        Genji { Secondary Fire: Off } \
        Roadhog { secondaryFireCooldown%: 50 secondaryFireMaximumTime%: 60 secondaryFireRechargeRate%: 70 Secondary Fire: Off Take a Breather: Off Take a Breather Cooldown Time: 80% } \
        Vendetta { Warding Stance Regen Scalar: 90% } \
        } } }";
    let program = internal::parse_in(source, &en);
    let emitted = internal::emit_in(&program, &en);
    for expected in [
        "No Automatic Fire: Off",
        "No Scope: Off",
        "Secondary Fire: Off",
        "secondaryFireCooldown%: 50",
        "secondaryFireMaximumTime%: 60",
        "secondaryFireRechargeRate%: 70",
        "Take a Breather: Off",
        "Take a Breather Cooldown Time: 80%",
        "Warding Stance Regen Scalar: 90%",
    ] {
        assert!(emitted.contains(expected), "{emitted} missing {expected}");
    }
    let reparsed = internal::parse_in(&emitted, &en);
    assert!(roundtrip::equivalent_wir(&program, &reparsed));
    assert!(
        program
            .semantic_issues(&catalog)
            .iter()
            .all(|issue| issue.kind != workshop_rs::rules::IncompletenessKind::RawSetting)
    );
}
