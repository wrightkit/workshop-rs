// Generated from the reviewed workshop-data hero settings export.
pub struct GeneratedHeroSettingName { pub hero: &'static str, pub key: &'static str, pub locales: &'static [(&'static str, &'static str)] }
impl GeneratedHeroSettingName {
    pub fn localized(&self, locale: &str) -> Option<&'static str> {
        self.locales
            .iter()
            .find(|(known, value)| {
                known.eq_ignore_ascii_case(locale) && !value.trim().is_empty() && !value.starts_with(' ')
            })
            .map(|(_, value)| *value)
    }
}
pub static GENERATED_HERO_SETTING_NAMES: &[GeneratedHeroSettingName] = &[
    GeneratedHeroSettingName {
        hero: "ana",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Sleep Dart Cooldown Time"), ("zh-CN", "麻醉镖冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Biotic Grenade Cooldown Time"), ("zh-CN", "生物手雷冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "enableAbility1",
        locales: &[("en-US", "Sleep Dart"), ("zh-CN", "麻醉镖")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "enableAbility2",
        locales: &[("en-US", "Biotic Grenade"), ("zh-CN", "生物手雷")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Nano Boost"), ("zh-CN", "战斗时终极技能充能速度 纳米激素")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Nano Boost"), ("zh-CN", "终极技能自动充能速度 纳米激素")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Nano Boost"), ("zh-CN", "终极技能充能速度（纳米激素）")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Nano Boost"), ("zh-CN", "终极技能（纳米激素）")],
    },
    GeneratedHeroSettingName {
        hero: "ana",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Inferno Rush Cooldown Time"), ("zh-CN", "怒炎冲冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Dancing Blaze Cooldown Time"), ("zh-CN", "熠闪舞冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "enableAbility1",
        locales: &[("en-US", "Inferno Rush"), ("zh-CN", "怒炎冲")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "enableAbility2",
        locales: &[("en-US", "Dancing Blaze"), ("zh-CN", "熠闪舞")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Vermillion Ascent"), ("zh-CN", "战斗时终极技能充能速度 朱羽焚")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Vermillion Ascent"), ("zh-CN", "终极技能自动充能速度 朱羽焚")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Vermillion Ascent"), ("zh-CN", "终极技能充能速度（朱羽焚）")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Vermillion Ascent"), ("zh-CN", "终极技能（朱羽焚）")],
    },
    GeneratedHeroSettingName {
        hero: "anran",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "ability1EnemyKb%",
        locales: &[("en-US", "Coach Gun Knockback Scalar Enemy"), ("zh-CN", "双筒猎枪击退距离（敌方）")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "ability1SelfKb%",
        locales: &[("en-US", "Coach Gun Knockback Scalar Self"), ("zh-CN", "双筒猎枪击退距离（自身）")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "ability2FuseTime%",
        locales: &[("en-US", "Dynamite Fuse Time Scalar"), ("zh-CN", "延时雷管引爆时间")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Coach Gun Cooldown Time"), ("zh-CN", "短筒猎枪冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Dynamite Cooldown Time"), ("zh-CN", "延时雷管冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "enableAbility1",
        locales: &[("en-US", "Coach Gun"), ("zh-CN", "短筒猎枪")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "enableAbility2",
        locales: &[("en-US", "Dynamite"), ("zh-CN", "延时雷管")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat B.O.B."), ("zh-CN", "战斗时终极技能充能速度 召唤鲍勃")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive B.O.B."), ("zh-CN", "终极技能自动充能速度 召唤鲍勃")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation B.O.B."), ("zh-CN", "终极技能充能速度（召唤鲍勃）")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability B.O.B."), ("zh-CN", "终极技能（召唤鲍勃）")],
    },
    GeneratedHeroSettingName {
        hero: "ashe",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Regenerative Burst Cooldown Time"), ("zh-CN", "愈合冲击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Immortality Field Cooldown Time"), ("zh-CN", "维生力场冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "enableAbility1",
        locales: &[("en-US", "Regenerative Burst"), ("zh-CN", "愈合冲击")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "enableAbility2",
        locales: &[("en-US", "Immortality Field"), ("zh-CN", "维生力场")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Amplification Matrix"), ("zh-CN", "战斗时终极技能充能速度 增幅矩阵")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Amplification Matrix"), ("zh-CN", "终极技能自动充能速度 增幅矩阵")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Amplification Matrix"), ("zh-CN", "终极技能充能速度（增幅矩阵）")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Amplification Matrix"), ("zh-CN", "终极技能（增幅矩阵）")],
    },
    GeneratedHeroSettingName {
        hero: "baptiste",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "secondaryFireKb%",
        locales: &[("en-US", "A-36 Tactical Grenade Knockback Scalar"), ("zh-CN", "A-36战术榴弹击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Reconfigure Cooldown Time"), ("zh-CN", "切换模式冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "ability2Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "enableAbility1",
        locales: &[("en-US", "Reconfigure"), ("zh-CN", "切换模式")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "enableAbility2",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "A-36 Tactical Grenade Cooldown Time"), ("zh-CN", "A-36战术榴弹冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "A-36 Tactical Grenade Energy Charge Rate"), ("zh-CN", "充能速度 A-36战术榴弹")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "A-36 Tactical Grenade Maximum Time"), ("zh-CN", "A-36战术榴弹最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "A-36 Tactical Grenade Recharge Rate"), ("zh-CN", "A-36战术榴弹充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "enableSecondaryFire",
        locales: &[("en-US", "A-36 Tactical Grenade"), ("zh-CN", "A-36战术榴弹")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Configuration: Artillery"), ("zh-CN", "战斗时终极技能充能速度 火炮模式")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Configuration: Artillery"), ("zh-CN", "终极技能自动充能速度 火炮模式")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Configuration: Artillery"), ("zh-CN", "终极技能充能速度（火炮模式）")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Configuration: Artillery"), ("zh-CN", "终极技能（火炮模式）")],
    },
    GeneratedHeroSettingName {
        hero: "bastion",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "shieldBashCooldown%",
        locales: &[("en-US", "Shield Bash Cooldown Time"), ("zh-CN", "能量盾击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "shieldBashKb%",
        locales: &[("en-US", "Shield Bash Knockback Scalar"), ("zh-CN", "能量盾击击退距离")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "ability1Kb%",
        locales: &[("en-US", "Whip Shot Knockback Scalar"), ("zh-CN", "流星飞锤击退距离")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Whip Shot Cooldown Time"), ("zh-CN", "流星飞锤冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Repair Pack Cooldown Time"), ("zh-CN", "恢复包冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "ability3Cooldown%",
        locales: &[("en-US", "Shield Bash Cooldown Time"), ("zh-CN", "能量盾击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "enableAbility1",
        locales: &[("en-US", "Whip Shot"), ("zh-CN", "流星飞锤")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "enableAbility2",
        locales: &[("en-US", "Repair Pack"), ("zh-CN", "恢复包")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "enableAbility3",
        locales: &[("en-US", "Shield Bash"), ("zh-CN", "能量盾击")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Barrier Shield Cooldown Time"), ("zh-CN", "屏障护盾冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Barrier Shield Energy Charge Rate"), ("zh-CN", "充能速度 屏障护盾")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Barrier Shield Maximum Time"), ("zh-CN", "屏障护盾最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Barrier Shield Recharge Rate"), ("zh-CN", "屏障护盾充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Barrier Shield"), ("zh-CN", "屏障护盾")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Rally"), ("zh-CN", "战斗时终极技能充能速度 集结号令")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Rally"), ("zh-CN", "终极技能自动充能速度 集结号令")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Rally"), ("zh-CN", "终极技能充能速度（集结号令）")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Rally"), ("zh-CN", "终极技能（集结号令）")],
    },
    GeneratedHeroSettingName {
        hero: "brigitte",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Combat Roll Cooldown Time"), ("zh-CN", "战术翻滚冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Flashbang Cooldown Time"), ("zh-CN", "闪光弹冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "enableAbility1",
        locales: &[("en-US", "Combat Roll"), ("zh-CN", "战术翻滚")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "enableAbility2",
        locales: &[("en-US", "Flashbang"), ("zh-CN", "闪光弹")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Deadeye"), ("zh-CN", "战斗时终极技能充能速度 神射手")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Deadeye"), ("zh-CN", "终极技能自动充能速度 神射手")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Deadeye"), ("zh-CN", "终极技能充能速度（神射手）")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Deadeye"), ("zh-CN", "终极技能（神射手）")],
    },
    GeneratedHeroSettingName {
        hero: "cassidy",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "assembleMechKb%",
        locales: &[("en-US", "[PH] Assemble Mech Knockback Scalar")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "ability1Kb%",
        locales: &[("en-US", "[PH] Propulsion Knockback Scalar")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "ability1MaxTime%",
        locales: &[("en-US", "Propulsors Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "ability1RechargeRate%",
        locales: &[("en-US", "Propulsors Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "ability3Kb%",
        locales: &[("en-US", "[PH] Skewer Knockback Scalar")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "spawnWithoutMech",
        locales: &[("en-US", "Spawn Without Mech"), ("zh-CN", "无机甲重生")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Propulsors Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Fusion Repeater Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "enableAbility1",
        locales: &[("en-US", "Propulsors")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "enableAbility2",
        locales: &[("en-US", "Fusion Repeater")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Power Barrier Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Power Barrier Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Power Barrier Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Power Barrier Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Power Barrier")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Limit Break")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Limit Break")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Limit Break")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Limit Break")],
    },
    GeneratedHeroSettingName {
        hero: "dmon",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "ability1ChargeRate%",
        locales: &[("en-US", "Power Block Charge Rate"), ("zh-CN", "悍猛格挡充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "secondaryFireKb%",
        locales: &[("en-US", "Rocket Punch Knockback Scalar"), ("zh-CN", "火箭重拳击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "ultKb%",
        locales: &[("en-US", "Meteor Strike Knockback Scalar"), ("zh-CN", "毁天灭地击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "ammoRegenerationTime%",
        locales: &[("en-US", "Ammunition Regeneration Time Scalar"), ("zh-CN", "弹药恢复时间速率")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Power Block Cooldown Time"), ("zh-CN", "悍猛格挡冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Seismic Slam Cooldown Time"), ("zh-CN", "裂地重拳冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "enableAbility1",
        locales: &[("en-US", "Power Block"), ("zh-CN", "悍猛格挡")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "enableAbility2",
        locales: &[("en-US", "Seismic Slam"), ("zh-CN", "裂地重拳")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Rocket Punch Cooldown Time"), ("zh-CN", "火箭重拳冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Rocket Punch Energy Charge Rate"), ("zh-CN", "充能速度 火箭重拳")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Rocket Punch Maximum Time"), ("zh-CN", "火箭重拳最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Rocket Punch Recharge Rate"), ("zh-CN", "火箭重拳充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Rocket Punch"), ("zh-CN", "火箭重拳")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Meteor Strike"), ("zh-CN", "战斗时终极技能充能速度 毁天灭地")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Meteor Strike"), ("zh-CN", "终极技能自动充能速度 毁天灭地")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Meteor Strike"), ("zh-CN", "终极技能充能速度（毁天灭地）")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Meteor Strike"), ("zh-CN", "终极技能（毁天灭地）")],
    },
    GeneratedHeroSettingName {
        hero: "doomfist",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "ability1Kb%",
        locales: &[("en-US", "Sonic Repulsors Knockback Scalar"), ("zh-CN", "音速斥力场击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "passiveHeal%",
        locales: &[("en-US", "Reconstruction Heal Scalar"), ("zh-CN", "护盾重构治疗量倍率")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "ultBarrierHealth%",
        locales: &[("en-US", "Ultimate Barrier Health Scalar Panopticon"), ("zh-CN", "终极技能屏障生命值倍率（全景牢笼）")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Sonic Repulsors Cooldown Time"), ("zh-CN", "音速斥力场冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Crystal Charge Cooldown Time"), ("zh-CN", "爆能水晶冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "enableAbility1",
        locales: &[("en-US", "Sonic Repulsors"), ("zh-CN", "音速斥力场")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "enableAbility2",
        locales: &[("en-US", "Crystal Charge"), ("zh-CN", "爆能水晶")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Barrier Array Cooldown Time"), ("zh-CN", "屏障阵列冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Barrier Array Energy Charge Rate"), ("zh-CN", "充能速度 屏障阵列")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Barrier Array Maximum Time"), ("zh-CN", "屏障阵列最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Barrier Array Recharge Rate"), ("zh-CN", "屏障阵列充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Barrier Array"), ("zh-CN", "屏障阵列")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Panopticon"), ("zh-CN", "战斗时终极技能充能速度 全景牢笼")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Panopticon"), ("zh-CN", "终极技能自动充能速度 全景牢笼")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Panopticon"), ("zh-CN", "终极技能充能速度（全景牢笼）")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Panopticon"), ("zh-CN", "终极技能（全景牢笼）")],
    },
    GeneratedHeroSettingName {
        hero: "domina",
        key: "enablePassive",
        locales: &[("en-US", "Reconstruction"), ("zh-CN", "护盾重构")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "ability1Kb%",
        locales: &[("en-US", "Boosters Knockback Scalar"), ("zh-CN", "推进器击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "callMechKb%",
        locales: &[("en-US", "Call Mech Knockback Scalar"), ("zh-CN", "呼叫机甲击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "selfDestructKb%",
        locales: &[("en-US", "Self Destruct Knockback Scalar"), ("zh-CN", "自毁击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "spawnWithoutMech",
        locales: &[("en-US", "Spawn Without Mech"), ("zh-CN", "无机甲重生")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Boosters Cooldown Time"), ("zh-CN", "推进器冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Micro Missiles Cooldown Time"), ("zh-CN", "微型飞弹冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "enableAbility1",
        locales: &[("en-US", "Boosters"), ("zh-CN", "推进器")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "enableAbility2",
        locales: &[("en-US", "Micro Missiles"), ("zh-CN", "微型飞弹")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Defense Matrix Cooldown Time"), ("zh-CN", "防御矩阵冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Defense Matrix Energy Charge Rate"), ("zh-CN", "充能速度 防御矩阵")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Defense Matrix Maximum Time"), ("zh-CN", "防御矩阵最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Defense Matrix Recharge Rate"), ("zh-CN", "防御矩阵充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Defense Matrix"), ("zh-CN", "防御矩阵")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Self-Destruct"), ("zh-CN", "战斗时终极技能充能速度 自毁")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Self-Destruct"), ("zh-CN", "终极技能自动充能速度 自毁")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Self-Destruct"), ("zh-CN", "终极技能充能速度（自毁）")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Self-Destruct"), ("zh-CN", "终极技能（自毁）")],
    },
    GeneratedHeroSettingName {
        hero: "dva",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Flight Cooldown Time"), ("zh-CN", "飞行冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Focusing Beam Cooldown Time"), ("zh-CN", "聚焦光线冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "enableAbility1",
        locales: &[("en-US", "Flight"), ("zh-CN", "飞行")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "enableAbility2",
        locales: &[("en-US", "Focusing Beam"), ("zh-CN", "聚焦光线")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Sticky Bombs Cooldown Time"), ("zh-CN", "黏性炸弹冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Sticky Bombs Energy Charge Rate"), ("zh-CN", "充能速度 黏性炸弹")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Sticky Bombs Maximum Time"), ("zh-CN", "黏性炸弹最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Sticky Bombs Recharge Rate"), ("zh-CN", "黏性炸弹充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Sticky Bombs"), ("zh-CN", "黏性炸弹")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Duplicate"), ("zh-CN", "战斗时终极技能充能速度 人格复制")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Duplicate"), ("zh-CN", "终极技能自动充能速度 人格复制")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Duplicate"), ("zh-CN", "终极技能充能速度（人格复制）")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Duplicate"), ("zh-CN", "终极技能（人格复制）")],
    },
    GeneratedHeroSettingName {
        hero: "echo",
        key: "enablePassive",
        locales: &[("en-US", "Glide"), ("zh-CN", "滑翔")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "ability2Kb%",
        locales: &[("en-US", "Cyber Frag Knockback Scalar"), ("zh-CN", "赛博手雷击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "ability1Duration%",
        locales: &[("en-US", "Siphon Blaster Duration Scalar"), ("zh-CN", "虹吸冲击枪持续时间倍率")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "ability1Heat%",
        locales: &[("en-US", "Siphon Blaster Heat Scalar"), ("zh-CN", "虹吸冲击枪热量倍率")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "ultKb%",
        locales: &[("en-US", "Override Protocol Knockback Scalar"), ("zh-CN", "覆盖协议击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Siphon Blaster Cooldown Time"), ("zh-CN", "虹吸冲击枪冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Cyber Frag Cooldown Time"), ("zh-CN", "赛博手雷冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "enableAbility1",
        locales: &[("en-US", "Siphon Blaster"), ("zh-CN", "虹吸冲击枪")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "enableAbility2",
        locales: &[("en-US", "Cyber Frag"), ("zh-CN", "赛博手雷")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Override Protocol"), ("zh-CN", "战斗时终极技能充能速度 覆盖协议")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Override Protocol"), ("zh-CN", "终极技能自动充能速度 覆盖协议")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Override Protocol"), ("zh-CN", "终极技能充能速度（覆盖协议）")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Override Protocol"), ("zh-CN", "终极技能（覆盖协议）")],
    },
    GeneratedHeroSettingName {
        hero: "emre",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "secondaryFireDuration%",
        locales: &[("en-US", "Take Aim Duration"), ("zh-CN", "瞄准射击持续时间")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "ability1Distance%",
        locales: &[("en-US", "Quick Dash Distance"), ("zh-CN", "疾冲距离")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "ability2Height%",
        locales: &[("en-US", "Updraft Height"), ("zh-CN", "上升气流高度")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Quick Dash Cooldown Time"), ("zh-CN", "疾冲冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Updraft Cooldown Time"), ("zh-CN", "上升气流冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "enableAbility1",
        locales: &[("en-US", "Quick Dash"), ("zh-CN", "疾冲")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "enableAbility2",
        locales: &[("en-US", "Updraft"), ("zh-CN", "上升气流")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Take Aim Cooldown Time"), ("zh-CN", "瞄准射击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Take Aim Energy Charge Rate"), ("zh-CN", "充能速度 瞄准射击")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Take Aim Maximum Time"), ("zh-CN", "瞄准射击最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Take Aim Recharge Rate"), ("zh-CN", "瞄准射击充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Take Aim"), ("zh-CN", "瞄准射击")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Bola Shot"), ("zh-CN", "战斗时终极技能充能速度 流星索")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Bola Shot"), ("zh-CN", "终极技能自动充能速度 流星索")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Bola Shot"), ("zh-CN", "终极技能充能速度（流星索）")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Bola Shot"), ("zh-CN", "终极技能（流星索）")],
    },
    GeneratedHeroSettingName {
        hero: "freja",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Swift Strike Cooldown Time"), ("zh-CN", "影冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Deflect Cooldown Time"), ("zh-CN", "闪冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "enableAbility1",
        locales: &[("en-US", "Swift Strike"), ("zh-CN", "影")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "enableAbility2",
        locales: &[("en-US", "Deflect"), ("zh-CN", "闪")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Dragonblade"), ("zh-CN", "战斗时终极技能充能速度 斩")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Dragonblade"), ("zh-CN", "终极技能自动充能速度 斩")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Dragonblade"), ("zh-CN", "终极技能充能速度（斩）")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Dragonblade"), ("zh-CN", "终极技能（斩）")],
    },
    GeneratedHeroSettingName {
        hero: "genji",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "solarEnergyMax%",
        locales: &[("en-US", "Solar Energy Maximum"), ("zh-CN", "太阳能上限")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "solarEnergyRecharge%",
        locales: &[("en-US", "Solar Energy Recharge Rate"), ("zh-CN", "太阳能充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Outburst Cooldown Time"), ("zh-CN", "烈日冲击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Healing Pylon Cooldown Time"), ("zh-CN", "治疗光塔冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "enableAbility1",
        locales: &[("en-US", "Outburst"), ("zh-CN", "烈日冲击")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "enableAbility2",
        locales: &[("en-US", "Healing Pylon"), ("zh-CN", "治疗光塔")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Captive Sun"), ("zh-CN", "战斗时终极技能充能速度 桎梏灼日")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Captive Sun"), ("zh-CN", "终极技能自动充能速度 桎梏灼日")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Captive Sun"), ("zh-CN", "终极技能充能速度（桎梏灼日）")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Captive Sun"), ("zh-CN", "终极技能（桎梏灼日）")],
    },
    GeneratedHeroSettingName {
        hero: "illari",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "enableRollOnly",
        locales: &[("en-US", "Roll Always Active"), ("zh-CN", "动力铁球始终激活")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "ability1Kb%",
        locales: &[("en-US", "Grappling Claw Knockback Scalar"), ("zh-CN", "工程抓钩击退距离")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "ultKb%",
        locales: &[("en-US", "Minefield Knockback Scalar"), ("zh-CN", "地雷禁区击退距离")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Roll Cooldown Time"), ("zh-CN", "动力铁球冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Adaptive Shield Cooldown Time"), ("zh-CN", "感应护盾冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "ability3Cooldown%",
        locales: &[("en-US", "Piledriver Cooldown Time"), ("zh-CN", "重力坠击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "enableAbility1",
        locales: &[("en-US", "Roll"), ("zh-CN", "动力铁球")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "enableAbility2",
        locales: &[("en-US", "Adaptive Shield"), ("zh-CN", "感应护盾")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "enableAbility3",
        locales: &[("en-US", "Piledriver"), ("zh-CN", "重力坠击")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Grappling Claw Cooldown Time"), ("zh-CN", "工程抓钩冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Grappling Claw Energy Charge Rate"), ("zh-CN", "充能速度 工程抓钩")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Grappling Claw Maximum Time"), ("zh-CN", "工程抓钩最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Grappling Claw Recharge Rate"), ("zh-CN", "工程抓钩充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Grappling Claw"), ("zh-CN", "工程抓钩")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Minefield"), ("zh-CN", "战斗时终极技能充能速度 地雷禁区")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Minefield"), ("zh-CN", "终极技能自动充能速度 地雷禁区")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Minefield"), ("zh-CN", "终极技能充能速度（地雷禁区）")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Minefield"), ("zh-CN", "终极技能（地雷禁区）")],
    },
    GeneratedHeroSettingName {
        hero: "wreckingBall",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "ability3Cooldown%",
        locales: &[("en-US", "Lunge Cooldown Time"), ("zh-CN", "“跃”冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "ability3Distance%",
        locales: &[("en-US", "Lunge Distance Scalar"), ("zh-CN", "“跃”距离设置")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "ability2Quantity%",
        locales: &[("en-US", "Storm Arrows Quantity"), ("zh-CN", "“岚”数量")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Sonic Arrow Cooldown Time"), ("zh-CN", "音冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Storm Arrows Cooldown Time"), ("zh-CN", "岚冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "ability3Cooldown%",
        locales: &[("en-US", "Lunge Cooldown Time"), ("zh-CN", "跃冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "enableAbility1",
        locales: &[("en-US", "Sonic Arrow"), ("zh-CN", "音")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "enableAbility2",
        locales: &[("en-US", "Storm Arrows"), ("zh-CN", "岚")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "enableAbility3",
        locales: &[("en-US", "Lunge"), ("zh-CN", "跃")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Dragonstrike"), ("zh-CN", "战斗时终极技能充能速度 竜")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Dragonstrike"), ("zh-CN", "终极技能自动充能速度 竜")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Dragonstrike"), ("zh-CN", "终极技能充能速度（竜）")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Dragonstrike"), ("zh-CN", "终极技能（竜）")],
    },
    GeneratedHeroSettingName {
        hero: "hanzo",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "primaryFireRange%",
        locales: &[("en-US", "Biotic Pawjectile Range"), ("zh-CN", "生物猫爪弹射程")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Frenetic Flight Maximum Time"), ("zh-CN", "咻咻飞最大时间")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "enablePassiveUnlimitedFuel",
        locales: &[("en-US", "Frenetic Flight Unlimited Fuel"), ("zh-CN", "咻咻飞无限燃料")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Lifeline Cooldown Time"), ("zh-CN", "救生索冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Purr Cooldown Time"), ("zh-CN", "呼噜噜冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "enableAbility1",
        locales: &[("en-US", "Lifeline"), ("zh-CN", "救生索")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "enableAbility2",
        locales: &[("en-US", "Purr"), ("zh-CN", "呼噜噜")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Frenetic Flight Cooldown Time"), ("zh-CN", "咻咻飞冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Frenetic Flight Energy Charge Rate"), ("zh-CN", "充能速度 咻咻飞")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Frenetic Flight Maximum Time"), ("zh-CN", "咻咻飞最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Frenetic Flight Recharge Rate"), ("zh-CN", "咻咻飞充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Frenetic Flight"), ("zh-CN", "咻咻飞")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Catnapper"), ("zh-CN", "战斗时终极技能充能速度 猫猫劫")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Catnapper"), ("zh-CN", "终极技能自动充能速度 猫猫劫")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Catnapper"), ("zh-CN", "终极技能充能速度（猫猫劫）")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Catnapper"), ("zh-CN", "终极技能（猫猫劫）")],
    },
    GeneratedHeroSettingName {
        hero: "jetpackCat",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Jagged Blade Gracie Cooldown Time"), ("zh-CN", "锯齿利刃（格雷西）冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "secondaryFireKb%",
        locales: &[("en-US", "Jagged Blade Knockback Scalar"), ("zh-CN", "锯齿利刃击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "secondaryFireRecallDelay%",
        locales: &[("en-US", "Jagged Blade Delay Before Automatic Recall"), ("zh-CN", "锯齿利刃自动召回延迟")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Commanding Shout Cooldown Time"), ("zh-CN", "命令怒吼冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Carnage Cooldown Time"), ("zh-CN", "血斩冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "enableAbility1",
        locales: &[("en-US", "Commanding Shout"), ("zh-CN", "命令怒吼")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "enableAbility2",
        locales: &[("en-US", "Carnage"), ("zh-CN", "血斩")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Jagged Blade Gracie Cooldown Time"), ("zh-CN", "锯齿利刃冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Jagged Blade Gracie Energy Charge Rate"), ("zh-CN", "充能速度 锯齿利刃")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Jagged Blade Gracie Maximum Time"), ("zh-CN", "锯齿利刃最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Jagged Blade Gracie Recharge Rate"), ("zh-CN", "锯齿利刃充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Jagged Blade Gracie"), ("zh-CN", "锯齿利刃")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Rampage"), ("zh-CN", "战斗时终极技能充能速度 轰翻天")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Rampage"), ("zh-CN", "终极技能自动充能速度 轰翻天")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Rampage"), ("zh-CN", "终极技能充能速度（轰翻天）")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Rampage"), ("zh-CN", "终极技能（轰翻天）")],
    },
    GeneratedHeroSettingName {
        hero: "junkerQueen",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "ability1Kb%",
        locales: &[("en-US", "Concussion Mine Knockback Scalar"), ("zh-CN", "震荡地雷击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "primaryFireKb%",
        locales: &[("en-US", "Frag Launcher Knockback Scalar"), ("zh-CN", "榴弹发射器击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Concussion Mine Cooldown Time"), ("zh-CN", "震荡地雷冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Steel Trap Cooldown Time"), ("zh-CN", "捕兽夹冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "enableAbility1",
        locales: &[("en-US", "Concussion Mine"), ("zh-CN", "震荡地雷")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "enableAbility2",
        locales: &[("en-US", "Steel Trap"), ("zh-CN", "捕兽夹")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat RIP-Tire"), ("zh-CN", "战斗时终极技能充能速度 炸弹轮胎")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive RIP-Tire"), ("zh-CN", "终极技能自动充能速度 炸弹轮胎")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation RIP-Tire"), ("zh-CN", "终极技能充能速度（炸弹轮胎）")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability RIP-Tire"), ("zh-CN", "终极技能（炸弹轮胎）")],
    },
    GeneratedHeroSettingName {
        hero: "junkrat",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "ability1Distance%",
        locales: &[("en-US", "Swift Step Distance Scalar"), ("zh-CN", "“瞬”距离倍率")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Swift Step Cooldown Time"), ("zh-CN", "瞬冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Protection Suzu Cooldown Time"), ("zh-CN", "铃冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "enableAbility1",
        locales: &[("en-US", "Swift Step"), ("zh-CN", "瞬")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "enableAbility2",
        locales: &[("en-US", "Protection Suzu"), ("zh-CN", "铃")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Kitsune Rush"), ("zh-CN", "战斗时终极技能充能速度 狐")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Kitsune Rush"), ("zh-CN", "终极技能自动充能速度 狐")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Kitsune Rush"), ("zh-CN", "终极技能充能速度（狐）")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Kitsune Rush"), ("zh-CN", "终极技能（狐）")],
    },
    GeneratedHeroSettingName {
        hero: "kiriko",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "secondaryFireKb%",
        locales: &[("en-US", "Soundwave Knockback Scalar"), ("zh-CN", "音波击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Crossfade Cooldown Time"), ("zh-CN", "切歌冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Amp It Up Cooldown Time"), ("zh-CN", "强音冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "enableAbility1",
        locales: &[("en-US", "Crossfade"), ("zh-CN", "切歌")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "enableAbility2",
        locales: &[("en-US", "Amp It Up"), ("zh-CN", "强音")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Soundwave Cooldown Time"), ("zh-CN", "音波冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Soundwave Energy Charge Rate"), ("zh-CN", "充能速度 音波")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Soundwave Maximum Time"), ("zh-CN", "音波最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Soundwave Recharge Rate"), ("zh-CN", "音波充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Soundwave"), ("zh-CN", "音波")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Sound Barrier"), ("zh-CN", "战斗时终极技能充能速度 音障")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Sound Barrier"), ("zh-CN", "终极技能自动充能速度 音障")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Sound Barrier"), ("zh-CN", "终极技能充能速度（音障）")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Sound Barrier"), ("zh-CN", "终极技能（音障）")],
    },
    GeneratedHeroSettingName {
        hero: "lucio",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "enablePrimaryFire",
        locales: &[("en-US", "Incendiary Chaingun"), ("zh-CN", "燃火链式机枪")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "primaryFireIgniteDamage",
        locales: &[("en-US", "Incendiary Chaingun Ignite Damage"), ("zh-CN", "燃火链式机枪点燃伤害")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "primaryFireIgniteDuration",
        locales: &[("en-US", "Incendiary Chaingun Ignite Duration"), ("zh-CN", "燃火链式机枪点燃持续时间")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "primaryFireIgniteRate",
        locales: &[("en-US", "Incendiary Chaingun Ignite Rate"), ("zh-CN", "燃火链式机枪点燃速度")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "ability1Kb%",
        locales: &[("en-US", "Overrun Knockback"), ("zh-CN", "蛮力冲撞击退")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "ability2Healing%",
        locales: &[("en-US", "Cardiac Overdrive Healing"), ("zh-CN", "心脏过载治疗量")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Overrun Cooldown Time"), ("zh-CN", "蛮力冲撞冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Cardiac Overdrive Cooldown Time"), ("zh-CN", "心脏过载冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "enableAbility1",
        locales: &[("en-US", "Overrun"), ("zh-CN", "蛮力冲撞")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "enableAbility2",
        locales: &[("en-US", "Cardiac Overdrive"), ("zh-CN", "心脏过载")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Volatile Chaingun Cooldown Time"), ("zh-CN", "爆烈链式机枪冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Volatile Chaingun Energy Charge Rate"), ("zh-CN", "充能速度 爆烈链式机枪")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Volatile Chaingun Maximum Time"), ("zh-CN", "爆烈链式机枪最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Volatile Chaingun Recharge Rate"), ("zh-CN", "爆烈链式机枪充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Volatile Chaingun"), ("zh-CN", "爆烈链式机枪")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Cage Fight"), ("zh-CN", "战斗时终极技能充能速度 笼中斗")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Cage Fight"), ("zh-CN", "终极技能自动充能速度 笼中斗")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Cage Fight"), ("zh-CN", "终极技能充能速度（笼中斗）")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Cage Fight"), ("zh-CN", "终极技能（笼中斗）")],
    },
    GeneratedHeroSettingName {
        hero: "mauga",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "ultFreezeMinimum%",
        locales: &[("en-US", "Blizzard Freeze Minimum"), ("zh-CN", "暴雪冰冻最小值")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "ultFreezeRate%",
        locales: &[("en-US", "Blizzard Freeze Rate Scalar"), ("zh-CN", "暴雪冰冻敌人的速度")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "enablePrimaryFireFreezeStack",
        locales: &[("en-US", "Freeze Stacking"), ("zh-CN", "冰冻叠加")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "primaryFireFreezeDuration%",
        locales: &[("en-US", "Weapon Freeze Duration Scalar"), ("zh-CN", "冰霜枪冰冻敌人的时长")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "primaryFireFreezeMinimum%",
        locales: &[("en-US", "Weapon Freeze Minimum"), ("zh-CN", "冰霜枪冰冻最小值")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "primaryFireFreezeRate%",
        locales: &[("en-US", "Weapon Freeze Rate Scalar"), ("zh-CN", "冰霜枪冰冻敌人的速度")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Cryo-Freeze Cooldown Time"), ("zh-CN", "急冻冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Ice Wall Cooldown Time"), ("zh-CN", "冰墙冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "enableAbility1",
        locales: &[("en-US", "Cryo-Freeze"), ("zh-CN", "急冻")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "enableAbility2",
        locales: &[("en-US", "Ice Wall"), ("zh-CN", "冰墙")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Blizzard"), ("zh-CN", "战斗时终极技能充能速度 暴雪")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Blizzard"), ("zh-CN", "终极技能自动充能速度 暴雪")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Blizzard"), ("zh-CN", "终极技能充能速度（暴雪）")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Blizzard"), ("zh-CN", "终极技能（暴雪）")],
    },
    GeneratedHeroSettingName {
        hero: "mei",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "weaponsEnabled",
        locales: &[("en-US", "Weapons Enabled"), ("zh-CN", "可用武器")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Guardian Angel Cooldown Time"), ("zh-CN", "守护天使冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Resurrect Cooldown Time"), ("zh-CN", "重生冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "enableAbility1",
        locales: &[("en-US", "Guardian Angel"), ("zh-CN", "守护天使")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "enableAbility2",
        locales: &[("en-US", "Resurrect"), ("zh-CN", "重生")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Valkyrie"), ("zh-CN", "战斗时终极技能充能速度 女武神")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Valkyrie"), ("zh-CN", "终极技能自动充能速度 女武神")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Valkyrie"), ("zh-CN", "终极技能充能速度（女武神）")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Valkyrie"), ("zh-CN", "终极技能（女武神）")],
    },
    GeneratedHeroSettingName {
        hero: "mercy",
        key: "enablePassive",
        locales: &[("en-US", "Sympathetic Recovery"), ("zh-CN", "同情愈疗")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "ability1Duration%",
        locales: &[("en-US", "Katashiro Return Duration Scalar"), ("zh-CN", "替魂纸人持续时间倍率")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Katashiro Return Cooldown Time"), ("zh-CN", "替魂纸人冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Binding Chain Cooldown Time"), ("zh-CN", "缚魂锁链冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "enableAbility1",
        locales: &[("en-US", "Katashiro Return"), ("zh-CN", "替魂纸人")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "enableAbility2",
        locales: &[("en-US", "Binding Chain"), ("zh-CN", "缚魂锁链")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Healing Kasa Cooldown Time"), ("zh-CN", "疗魂斗笠冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Healing Kasa Energy Charge Rate"), ("zh-CN", "充能速度 疗魂斗笠")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Healing Kasa Maximum Time"), ("zh-CN", "疗魂斗笠最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Healing Kasa Recharge Rate"), ("zh-CN", "疗魂斗笠充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Healing Kasa"), ("zh-CN", "疗魂斗笠")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Kekkai Sanctuary"), ("zh-CN", "战斗时终极技能充能速度 护魂结界")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Kekkai Sanctuary"), ("zh-CN", "终极技能自动充能速度 护魂结界")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Kekkai Sanctuary"), ("zh-CN", "终极技能充能速度（护魂结界）")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Kekkai Sanctuary"), ("zh-CN", "终极技能（护魂结界）")],
    },
    GeneratedHeroSettingName {
        hero: "mizuki",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "ability2MaxDamage%",
        locales: &[("en-US", "Biotic Orb Max Damage Scalar"), ("zh-CN", "生化之球伤害最大值")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "ability2MaxHealing%",
        locales: &[("en-US", "Biotic Orb Max Healing Scalar"), ("zh-CN", "生化之球治疗最大值")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "primaryFireMaximumTime%",
        locales: &[("en-US", "Biotic Energy Maximum"), ("zh-CN", "生化能量上限")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "primaryFireRechargeRate%",
        locales: &[("en-US", "Biotic Energy Recharge Rate"), ("zh-CN", "生化能量恢复速度")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Fade Cooldown Time"), ("zh-CN", "消散冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Biotic Orb Cooldown Time"), ("zh-CN", "生化之球冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "enableAbility1",
        locales: &[("en-US", "Fade"), ("zh-CN", "消散")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "enableAbility2",
        locales: &[("en-US", "Biotic Orb"), ("zh-CN", "生化之球")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Coalescence"), ("zh-CN", "战斗时终极技能充能速度 聚合射线")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Coalescence"), ("zh-CN", "终极技能自动充能速度 聚合射线")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Coalescence"), ("zh-CN", "终极技能充能速度（聚合射线）")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Coalescence"), ("zh-CN", "终极技能（聚合射线）")],
    },
    GeneratedHeroSettingName {
        hero: "moira",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Fortify Cooldown Time"), ("zh-CN", "强固防御冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Javelin Spin Cooldown Time"), ("zh-CN", "标枪旋击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "enableAbility1",
        locales: &[("en-US", "Fortify"), ("zh-CN", "强固防御")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "enableAbility2",
        locales: &[("en-US", "Javelin Spin"), ("zh-CN", "标枪旋击")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Energy Javelin Cooldown Time"), ("zh-CN", "能量标枪冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Energy Javelin Energy Charge Rate"), ("zh-CN", "充能速度 能量标枪")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Energy Javelin Maximum Time"), ("zh-CN", "能量标枪最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Energy Javelin Recharge Rate"), ("zh-CN", "能量标枪充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Energy Javelin"), ("zh-CN", "能量标枪")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Terra Surge"), ("zh-CN", "战斗时终极技能充能速度 撼地猛刺")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Terra Surge"), ("zh-CN", "终极技能自动充能速度 撼地猛刺")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Terra Surge"), ("zh-CN", "终极技能充能速度（撼地猛刺）")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Terra Surge"), ("zh-CN", "终极技能（撼地猛刺）")],
    },
    GeneratedHeroSettingName {
        hero: "orisa",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "ability2Kb%",
        locales: &[("en-US", "Concussive Blast Knockback Scalar"), ("zh-CN", "震荡冲击击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "enablePassiveUnlimitedFuel",
        locales: &[("en-US", "Hover Jets Unlimited Fuel"), ("zh-CN", "悬浮背包无限燃料")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "passiveVerticalSpeed%",
        locales: &[("en-US", "Hover Jets Vertical Speed Scalar"), ("zh-CN", "悬浮背包垂直速度")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "passiveMaximumTime%",
        locales: &[("en-US", "Jump Jet Acceleration Scalar"), ("zh-CN", "推进背包加速")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "passiveExtraFuel%",
        locales: &[("en-US", "Hover Jets Extra Fuel Scalar"), ("zh-CN", "悬浮背包额外燃料倍率")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "passiveRechargeRate%",
        locales: &[("en-US", "Hover Jets Recharge Rate"), ("zh-CN", "悬浮背包充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "passiveMaxTime%",
        locales: &[("en-US", "Hover Jets Maximum Time"), ("zh-CN", "悬浮背包最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "ability1Acceleration%",
        locales: &[("en-US", "Jump Jet Acceleration Scalar"), ("zh-CN", "推进背包加速")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "ability1RefuelScalar",
        locales: &[("en-US", "Jump Jet Refuel Scalar"), ("zh-CN", "推进背包燃料补充倍率")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "primaryFireKb%",
        locales: &[("en-US", "Rocket Launcher Knockback Scalar"), ("zh-CN", "火箭发射器击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Jump Jet Cooldown Time"), ("zh-CN", "推进背包冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Concussive Blast Cooldown Time"), ("zh-CN", "震荡冲击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "enableAbility1",
        locales: &[("en-US", "Jump Jet"), ("zh-CN", "推进背包")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "enableAbility2",
        locales: &[("en-US", "Concussive Blast"), ("zh-CN", "震荡冲击")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Jet Dash Cooldown Time"), ("zh-CN", "疾冲背包冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Jet Dash Energy Charge Rate"), ("zh-CN", "充能速度 疾冲背包")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Jet Dash Maximum Time"), ("zh-CN", "疾冲背包最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Jet Dash Recharge Rate"), ("zh-CN", "疾冲背包充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Jet Dash"), ("zh-CN", "疾冲背包")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Barrage"), ("zh-CN", "战斗时终极技能充能速度 火箭弹幕")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Barrage"), ("zh-CN", "终极技能自动充能速度 火箭弹幕")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Barrage"), ("zh-CN", "终极技能充能速度（火箭弹幕）")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Barrage"), ("zh-CN", "终极技能（火箭弹幕）")],
    },
    GeneratedHeroSettingName {
        hero: "pharah",
        key: "enablePassive",
        locales: &[("en-US", "Hover Jets"), ("zh-CN", "悬浮背包")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Wraith Form Cooldown Time"), ("zh-CN", "幽灵形态冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Shadow Step Cooldown Time"), ("zh-CN", "暗影步冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "enableAbility1",
        locales: &[("en-US", "Wraith Form"), ("zh-CN", "幽灵形态")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "enableAbility2",
        locales: &[("en-US", "Shadow Step"), ("zh-CN", "暗影步")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Death Blossom"), ("zh-CN", "战斗时终极技能充能速度 死亡绽放")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Death Blossom"), ("zh-CN", "终极技能自动充能速度 死亡绽放")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Death Blossom"), ("zh-CN", "终极技能充能速度（死亡绽放）")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Death Blossom"), ("zh-CN", "终极技能（死亡绽放）")],
    },
    GeneratedHeroSettingName {
        hero: "reaper",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "ability1Kb%",
        locales: &[("en-US", "Charge Knockback Scalar"), ("zh-CN", "冲锋击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "primaryFireKb%",
        locales: &[("en-US", "Rocket Hammer Knockback Scalar"), ("zh-CN", "火箭重锤击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Charge Cooldown Time"), ("zh-CN", "冲锋冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Fire Strike Cooldown Time"), ("zh-CN", "烈焰打击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "enableAbility1",
        locales: &[("en-US", "Charge"), ("zh-CN", "冲锋")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "enableAbility2",
        locales: &[("en-US", "Fire Strike"), ("zh-CN", "烈焰打击")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Barrier Field Cooldown Time"), ("zh-CN", "屏障力场冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Barrier Field Energy Charge Rate"), ("zh-CN", "充能速度 屏障力场")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Barrier Field Maximum Time"), ("zh-CN", "屏障力场最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Barrier Field Recharge Rate"), ("zh-CN", "屏障力场充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Barrier Field"), ("zh-CN", "屏障力场")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Earthshatter"), ("zh-CN", "战斗时终极技能充能速度 裂地猛击")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Earthshatter"), ("zh-CN", "终极技能自动充能速度 裂地猛击")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Earthshatter"), ("zh-CN", "终极技能充能速度（裂地猛击）")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Earthshatter"), ("zh-CN", "终极技能（裂地猛击）")],
    },
    GeneratedHeroSettingName {
        hero: "reinhardt",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "ultKb%",
        locales: &[("en-US", "Whole Hog Knockback Scalar"), ("zh-CN", "鸡飞狗跳击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Chain Hook Cooldown Time"), ("zh-CN", "链钩冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Take a Breather Cooldown Time"), ("zh-CN", "呼吸器冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "enableAbility1",
        locales: &[("en-US", "Chain Hook"), ("zh-CN", "链钩")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "enableAbility2",
        locales: &[("en-US", "Take a Breather"), ("zh-CN", "呼吸器")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Whole Hog"), ("zh-CN", "战斗时终极技能充能速度 鸡飞狗跳")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Whole Hog"), ("zh-CN", "终极技能自动充能速度 鸡飞狗跳")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Whole Hog"), ("zh-CN", "终极技能充能速度（鸡飞狗跳）")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Whole Hog"), ("zh-CN", "终极技能（鸡飞狗跳）")],
    },
    GeneratedHeroSettingName {
        hero: "roadhog",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "ability1Distance%",
        locales: &[("en-US", "Evade Distance Scalar"), ("zh-CN", "机动闪避距离倍率")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "ability2Kb%",
        locales: &[("en-US", "Joyride Knockback Scalar"), ("zh-CN", "纵情狂飙击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "ability2Duration%",
        locales: &[("en-US", "Joyride Duration Scalar"), ("zh-CN", "纵情狂飙持续时间倍率")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "ability2Speed%",
        locales: &[("en-US", "Joyride Speed Scalar"), ("zh-CN", "纵情狂飙速度倍率")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Evade Cooldown Time"), ("zh-CN", "机动闪避冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Joyride Cooldown Time"), ("zh-CN", "纵情狂飙冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "enableAbility1",
        locales: &[("en-US", "Evade"), ("zh-CN", "机动闪避")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "enableAbility2",
        locales: &[("en-US", "Joyride"), ("zh-CN", "纵情狂飙")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Execution Cooldown Time"), ("zh-CN", "交叉枪决冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Execution Energy Charge Rate"), ("zh-CN", "充能速度 交叉枪决")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Execution Maximum Time"), ("zh-CN", "交叉枪决最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Execution Recharge Rate"), ("zh-CN", "交叉枪决充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Execution"), ("zh-CN", "交叉枪决")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Satsuriku Spree"), ("zh-CN", "战斗时终极技能充能速度 杀戮狂宴")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Satsuriku Spree"), ("zh-CN", "终极技能自动充能速度 杀戮狂宴")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Satsuriku Spree"), ("zh-CN", "终极技能充能速度（杀戮狂宴）")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Satsuriku Spree"), ("zh-CN", "终极技能（杀戮狂宴）")],
    },
    GeneratedHeroSettingName {
        hero: "shion",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "ability2Kb%",
        locales: &[("en-US", "Tremor Charge Knockback Scalar"), ("zh-CN", "震地手雷击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Anchor Drone Cooldown Time"), ("zh-CN", "锚点无人机冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Tremor Charge Cooldown Time"), ("zh-CN", "震地手雷冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "enableAbility1",
        locales: &[("en-US", "Anchor Drone"), ("zh-CN", "锚点无人机")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "enableAbility2",
        locales: &[("en-US", "Tremor Charge"), ("zh-CN", "震地手雷")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Tracking Shot Cooldown Time"), ("zh-CN", "追踪弹冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Tracking Shot Energy Charge Rate"), ("zh-CN", "充能速度 追踪弹")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Tracking Shot Maximum Time"), ("zh-CN", "追踪弹最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Tracking Shot Recharge Rate"), ("zh-CN", "追踪弹充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Tracking Shot"), ("zh-CN", "追踪弹")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Trailblazer"), ("zh-CN", "战斗时终极技能充能速度 开路先锋")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Trailblazer"), ("zh-CN", "终极技能自动充能速度 开路先锋")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Trailblazer"), ("zh-CN", "终极技能充能速度（开路先锋）")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Trailblazer"), ("zh-CN", "终极技能（开路先锋）")],
    },
    GeneratedHeroSettingName {
        hero: "sierra",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "ability2Kb%",
        locales: &[("en-US", "Accretion Knockback Scalar"), ("zh-CN", "质量吸附击退距离")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Kinetic Grasp Cooldown Time"), ("zh-CN", "动能俘获冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Accretion Cooldown Time"), ("zh-CN", "质量吸附冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "enableAbility1",
        locales: &[("en-US", "Kinetic Grasp"), ("zh-CN", "动能俘获")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "enableAbility2",
        locales: &[("en-US", "Accretion"), ("zh-CN", "质量吸附")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Experimental Barrier Cooldown Time"), ("zh-CN", "实验屏障冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Experimental Barrier Energy Charge Rate"), ("zh-CN", "充能速度 实验屏障")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Experimental Barrier Maximum Time"), ("zh-CN", "实验屏障最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Experimental Barrier Recharge Rate"), ("zh-CN", "实验屏障充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Experimental Barrier"), ("zh-CN", "实验屏障")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Gravitic Flux"), ("zh-CN", "战斗时终极技能充能速度 引力乱流")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Gravitic Flux"), ("zh-CN", "终极技能自动充能速度 引力乱流")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Gravitic Flux"), ("zh-CN", "终极技能充能速度（引力乱流）")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Gravitic Flux"), ("zh-CN", "终极技能（引力乱流）")],
    },
    GeneratedHeroSettingName {
        hero: "sigma",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Power Slide Cooldown Time"), ("zh-CN", "机动滑铲冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Disruptor Shot Cooldown Time"), ("zh-CN", "干扰弹冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "enableAbility1",
        locales: &[("en-US", "Power Slide"), ("zh-CN", "机动滑铲")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "enableAbility2",
        locales: &[("en-US", "Disruptor Shot"), ("zh-CN", "干扰弹")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Charged Shot Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Charged Shot Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Charged Shot Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Charged Shot Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Charged Shot")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Overclock"), ("zh-CN", "战斗时终极技能充能速度 机体超频")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Overclock"), ("zh-CN", "终极技能自动充能速度 机体超频")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Overclock"), ("zh-CN", "终极技能充能速度（机体超频）")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Overclock"), ("zh-CN", "终极技能（机体超频）")],
    },
    GeneratedHeroSettingName {
        hero: "sojourn",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "secondaryFireKb%",
        locales: &[("en-US", "Helix Rockets Knockback Scalar"), ("zh-CN", "螺旋飞弹击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Sprint Cooldown Time"), ("zh-CN", "疾跑冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Biotic Field Cooldown Time"), ("zh-CN", "生物力场冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "enableAbility1",
        locales: &[("en-US", "Sprint"), ("zh-CN", "疾跑")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "enableAbility2",
        locales: &[("en-US", "Biotic Field"), ("zh-CN", "生物力场")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Helix Rockets Cooldown Time"), ("zh-CN", "螺旋飞弹冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Helix Rockets Energy Charge Rate"), ("zh-CN", "充能速度 螺旋飞弹")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Helix Rockets Maximum Time"), ("zh-CN", "螺旋飞弹最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Helix Rockets Recharge Rate"), ("zh-CN", "螺旋飞弹充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Helix Rockets"), ("zh-CN", "螺旋飞弹")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Tactical Visor"), ("zh-CN", "战斗时终极技能充能速度 战术目镜")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Tactical Visor"), ("zh-CN", "终极技能自动充能速度 战术目镜")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Tactical Visor"), ("zh-CN", "终极技能充能速度（战术目镜）")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Tactical Visor"), ("zh-CN", "终极技能（战术目镜）")],
    },
    GeneratedHeroSettingName {
        hero: "soldier",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Virus Cooldown Time"), ("zh-CN", "病毒冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Translocator Cooldown Time"), ("zh-CN", "位移传动冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "enableAbility1",
        locales: &[("en-US", "Virus"), ("zh-CN", "病毒")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "enableAbility2",
        locales: &[("en-US", "Translocator"), ("zh-CN", "位移传动")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Hack Cooldown Time"), ("zh-CN", "黑客入侵冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Hack Energy Charge Rate"), ("zh-CN", "充能速度 黑客入侵")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Hack Maximum Time"), ("zh-CN", "黑客入侵最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Hack Recharge Rate"), ("zh-CN", "黑客入侵充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Hack"), ("zh-CN", "黑客入侵")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat EMP"), ("zh-CN", "战斗时终极技能充能速度 电磁脉冲")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive EMP"), ("zh-CN", "终极技能自动充能速度 电磁脉冲")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation EMP"), ("zh-CN", "终极技能充能速度（电磁脉冲）")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability EMP"), ("zh-CN", "终极技能（电磁脉冲）")],
    },
    GeneratedHeroSettingName {
        hero: "sombra",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Sentry Turret Cooldown Time"), ("zh-CN", "哨戒炮冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Teleporter Cooldown Time"), ("zh-CN", "传送面板冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "enableAbility1",
        locales: &[("en-US", "Sentry Turret"), ("zh-CN", "哨戒炮")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "enableAbility2",
        locales: &[("en-US", "Teleporter"), ("zh-CN", "传送面板")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Photon Barrier"), ("zh-CN", "战斗时终极技能充能速度 光子屏障")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Photon Barrier"), ("zh-CN", "终极技能自动充能速度 光子屏障")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Photon Barrier"), ("zh-CN", "终极技能充能速度（光子屏障）")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Photon Barrier"), ("zh-CN", "终极技能（光子屏障）")],
    },
    GeneratedHeroSettingName {
        hero: "symmetra",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "ability2Duration%",
        locales: &[("en-US", "Overload Duration Scalar"), ("zh-CN", "热力过载持续时间")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "weaponsEnabled",
        locales: &[("en-US", "Weapons Enabled"), ("zh-CN", "可用武器")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Deploy Turret Cooldown Time"), ("zh-CN", "部署炮台冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Overload Cooldown Time"), ("zh-CN", "热力过载冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "enableAbility1",
        locales: &[("en-US", "Deploy Turret"), ("zh-CN", "部署炮台")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "enableAbility2",
        locales: &[("en-US", "Overload"), ("zh-CN", "热力过载")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Molten Core"), ("zh-CN", "战斗时终极技能充能速度 熔火核心")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Molten Core"), ("zh-CN", "终极技能自动充能速度 熔火核心")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Molten Core"), ("zh-CN", "终极技能充能速度（熔火核心）")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Molten Core"), ("zh-CN", "终极技能（熔火核心）")],
    },
    GeneratedHeroSettingName {
        hero: "torbjorn",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Blink Cooldown Time"), ("zh-CN", "闪现冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Recall Cooldown Time"), ("zh-CN", "闪回冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "enableAbility1",
        locales: &[("en-US", "Blink"), ("zh-CN", "闪现")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "enableAbility2",
        locales: &[("en-US", "Recall"), ("zh-CN", "闪回")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Pulse Bomb"), ("zh-CN", "战斗时终极技能充能速度 脉冲炸弹")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Pulse Bomb"), ("zh-CN", "终极技能自动充能速度 脉冲炸弹")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Pulse Bomb"), ("zh-CN", "终极技能充能速度（脉冲炸弹）")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Pulse Bomb"), ("zh-CN", "终极技能（脉冲炸弹）")],
    },
    GeneratedHeroSettingName {
        hero: "tracer",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Grappling Hook Cooldown Time"), ("zh-CN", "抓钩冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Venom Mine Cooldown Time"), ("zh-CN", "剧毒诡雷冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "enableAbility1",
        locales: &[("en-US", "Grappling Hook"), ("zh-CN", "抓钩")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "enableAbility2",
        locales: &[("en-US", "Venom Mine"), ("zh-CN", "剧毒诡雷")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Infra-Sight"), ("zh-CN", "战斗时终极技能充能速度 红外侦测")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Infra-Sight"), ("zh-CN", "终极技能自动充能速度 红外侦测")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Infra-Sight"), ("zh-CN", "终极技能充能速度（红外侦测）")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Infra-Sight"), ("zh-CN", "终极技能（红外侦测）")],
    },
    GeneratedHeroSettingName {
        hero: "widowmaker",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "ability1Acceleration%",
        locales: &[("en-US", "Jump Pack Acceleration Scalar"), ("zh-CN", "喷射背包加速")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "ability1Kb%",
        locales: &[("en-US", "Jump Pack Knockback Scalar"), ("zh-CN", "喷射背包击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "ultKb%",
        locales: &[("en-US", "Primal Rage Melee Knockback Scalar"), ("zh-CN", "原始暴怒近身攻击击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Jump Pack Cooldown Time"), ("zh-CN", "喷射背包冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Barrier Projector Cooldown Time"), ("zh-CN", "屏障发射器冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "enableAbility1",
        locales: &[("en-US", "Jump Pack"), ("zh-CN", "喷射背包")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "enableAbility2",
        locales: &[("en-US", "Barrier Projector"), ("zh-CN", "屏障发射器")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Primal Rage"), ("zh-CN", "战斗时终极技能充能速度 原始暴怒")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Primal Rage"), ("zh-CN", "终极技能自动充能速度 原始暴怒")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Primal Rage"), ("zh-CN", "终极技能充能速度（原始暴怒）")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Primal Rage"), ("zh-CN", "终极技能（原始暴怒）")],
    },
    GeneratedHeroSettingName {
        hero: "winston",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "secondaryFireKb%",
        locales: &[("en-US", "Particle Cannon Secondary Knockback Scalar"), ("zh-CN", "粒子炮辅助攻击击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Particle Barrier Cooldown Time"), ("zh-CN", "粒子屏障冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Projected Barrier Cooldown Time"), ("zh-CN", "投射屏障冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "enableAbility1",
        locales: &[("en-US", "Particle Barrier"), ("zh-CN", "粒子屏障")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "enableAbility2",
        locales: &[("en-US", "Projected Barrier"), ("zh-CN", "投射屏障")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Graviton Surge"), ("zh-CN", "战斗时终极技能充能速度 重力喷涌")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Graviton Surge"), ("zh-CN", "终极技能自动充能速度 重力喷涌")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Graviton Surge"), ("zh-CN", "终极技能充能速度（重力喷涌）")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Graviton Surge"), ("zh-CN", "终极技能（重力喷涌）")],
    },
    GeneratedHeroSettingName {
        hero: "zarya",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Orb of Harmony Cooldown Time"), ("zh-CN", "谐冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Orb of Discord Cooldown Time"), ("zh-CN", "乱冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "enableAbility1",
        locales: &[("en-US", "Orb of Harmony"), ("zh-CN", "谐")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "enableAbility2",
        locales: &[("en-US", "Orb of Discord"), ("zh-CN", "乱")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", " Energy Charge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", " Maximum Time")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", " Recharge Rate")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "enableSecondaryFire",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Transcendence"), ("zh-CN", "战斗时终极技能充能速度 圣")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Transcendence"), ("zh-CN", "终极技能自动充能速度 圣")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Transcendence"), ("zh-CN", "终极技能充能速度（圣）")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Transcendence"), ("zh-CN", "终极技能（圣）")],
    },
    GeneratedHeroSettingName {
        hero: "zenyatta",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "secondaryFireAlternateForm",
        locales: &[("en-US", "Block Nemesis Form"), ("zh-CN", "铁臂（天罚形态）")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Nemesis Form Cooldown Time"), ("zh-CN", "天罚形态冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Ravenous Vortex Cooldown Time"), ("zh-CN", "吞噬漩涡冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "enableAbility1",
        locales: &[("en-US", "Nemesis Form"), ("zh-CN", "天罚形态")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "enableAbility2",
        locales: &[("en-US", "Ravenous Vortex"), ("zh-CN", "吞噬漩涡")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Void Barrier Omnic Form Cooldown Time"), ("zh-CN", "虚空屏障（智械形态）冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Void Barrier Omnic Form Energy Charge Rate"), ("zh-CN", "充能速度 虚空屏障（智械形态）")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Void Barrier Omnic Form Maximum Time"), ("zh-CN", "虚空屏障（智械形态）最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Void Barrier Omnic Form Recharge Rate"), ("zh-CN", "虚空屏障（智械形态）充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Void Barrier Omnic Form"), ("zh-CN", "虚空屏障（智械形态）")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Annihilation"), ("zh-CN", "战斗时终极技能充能速度 诛")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Annihilation"), ("zh-CN", "终极技能自动充能速度 诛")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Annihilation"), ("zh-CN", "终极技能充能速度（诛）")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Annihilation"), ("zh-CN", "终极技能（诛）")],
    },
    GeneratedHeroSettingName {
        hero: "ramattra",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "ultHealth%",
        locales: &[("en-US", "Tree of Life Health"), ("zh-CN", "生命之树生命值")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "ability1Health%",
        locales: &[("en-US", "Rejuvenating Dash Healing"), ("zh-CN", "回春疾行治疗量")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "secondaryFireHealth%",
        locales: &[("en-US", "Petal Platform Health"), ("zh-CN", "花瓣平台生命值")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "primaryFireRange%",
        locales: &[("en-US", "Life Grip and Healing Blossom Range"), ("zh-CN", "生命之握和愈疗灵花射程")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "weaponsEnabled",
        locales: &[("en-US", "Weapons Enabled"), ("zh-CN", "可用武器")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Rejuvenating Dash Cooldown Time"), ("zh-CN", "回春疾行冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Life Grip Cooldown Time"), ("zh-CN", "生命之握冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "enableAbility1",
        locales: &[("en-US", "Rejuvenating Dash"), ("zh-CN", "回春疾行")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "enableAbility2",
        locales: &[("en-US", "Life Grip"), ("zh-CN", "生命之握")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Petal Platform Cooldown Time"), ("zh-CN", "花瓣平台冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Petal Platform Energy Charge Rate"), ("zh-CN", "充能速度 花瓣平台")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Petal Platform Maximum Time"), ("zh-CN", "花瓣平台最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Petal Platform Recharge Rate"), ("zh-CN", "花瓣平台充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Petal Platform"), ("zh-CN", "花瓣平台")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Tree of Life"), ("zh-CN", "战斗时终极技能充能速度 生命之树")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Tree of Life"), ("zh-CN", "终极技能自动充能速度 生命之树")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Tree of Life"), ("zh-CN", "终极技能充能速度（生命之树）")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Tree of Life"), ("zh-CN", "终极技能（生命之树）")],
    },
    GeneratedHeroSettingName {
        hero: "lifeweaver",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "ability1Duration%",
        locales: &[("en-US", "Burrow Duration Scalar"), ("zh-CN", "钻地持续时间倍率")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Burrow Cooldown Time"), ("zh-CN", "钻地冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "ability2Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "enableAbility1",
        locales: &[("en-US", "Burrow"), ("zh-CN", "钻地")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "enableAbility2",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Drill Dash Cooldown Time"), ("zh-CN", "钻头突刺冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Drill Dash Energy Charge Rate"), ("zh-CN", "充能速度 钻头突刺")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Drill Dash Maximum Time"), ("zh-CN", "钻头突刺最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Drill Dash Recharge Rate"), ("zh-CN", "钻头突刺充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Drill Dash"), ("zh-CN", "钻头突刺")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Tectonic Shock"), ("zh-CN", "战斗时终极技能充能速度 地壳震击")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Tectonic Shock"), ("zh-CN", "终极技能自动充能速度 地壳震击")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Tectonic Shock"), ("zh-CN", "终极技能充能速度（地壳震击）")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Tectonic Shock"), ("zh-CN", "终极技能（地壳震击）")],
    },
    GeneratedHeroSettingName {
        hero: "venture",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "ability1Duration%",
        locales: &[("en-US", "Glide Boost Duration Scalar"), ("zh-CN", "滑翔推进持续时间倍率")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Glide Boost Cooldown Time"), ("zh-CN", "滑翔推进冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Hyper Ring Cooldown Time"), ("zh-CN", "超能环域冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "enableAbility1",
        locales: &[("en-US", "Glide Boost"), ("zh-CN", "滑翔推进")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "enableAbility2",
        locales: &[("en-US", "Hyper Ring"), ("zh-CN", "超能环域")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Pulsar Torpedoes Cooldown Time"), ("zh-CN", "脉冲星飞雷冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Pulsar Torpedoes Energy Charge Rate"), ("zh-CN", "充能速度 脉冲星飞雷")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Pulsar Torpedoes Maximum Time"), ("zh-CN", "脉冲星飞雷最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Pulsar Torpedoes Recharge Rate"), ("zh-CN", "脉冲星飞雷充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Pulsar Torpedoes"), ("zh-CN", "脉冲星飞雷")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Orbital Ray"), ("zh-CN", "战斗时终极技能充能速度 轨道射线")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Orbital Ray"), ("zh-CN", "终极技能自动充能速度 轨道射线")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Orbital Ray"), ("zh-CN", "终极技能充能速度（轨道射线）")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Orbital Ray"), ("zh-CN", "终极技能（轨道射线）")],
    },
    GeneratedHeroSettingName {
        hero: "juno",
        key: "enablePassive",
        locales: &[("en-US", "Martian Overboots"), ("zh-CN", "火星套靴")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "ability2Kb%",
        locales: &[("en-US", "Jagged Wall Knockback"), ("zh-CN", "尖刺墙击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "ability2Health%",
        locales: &[("en-US", "Jagged Wall Health"), ("zh-CN", "尖刺墙生命值倍率")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "secondaryFireMovementSpeedPenalty%",
        locales: &[("en-US", "Spike Guard Movement Speed Penalty")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "secondaryFireCost%",
        locales: &[("en-US", "Spike Guard Resource Cost"), ("zh-CN", "尖刺护体资源消耗倍率")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "secondaryFireRegen%",
        locales: &[("en-US", "Spike Guard Resource Regeneration"), ("zh-CN", "尖刺护体资源恢复倍率")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "ability1Distance%",
        locales: &[("en-US", "Violent Leap Distance"), ("zh-CN", "狂跃距离倍率")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Violent Leap Cooldown Time"), ("zh-CN", "狂跃冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Jagged Wall Cooldown Time"), ("zh-CN", "尖刺墙冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "enableAbility1",
        locales: &[("en-US", "Violent Leap"), ("zh-CN", "狂跃")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "enableAbility2",
        locales: &[("en-US", "Jagged Wall"), ("zh-CN", "尖刺墙")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Spike Guard Cooldown Time"), ("zh-CN", "尖刺护体冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Spike Guard Energy Charge Rate"), ("zh-CN", "充能速度 尖刺护体")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Spike Guard Maximum Time"), ("zh-CN", "尖刺护体最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Spike Guard Recharge Rate"), ("zh-CN", "尖刺护体充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Spike Guard"), ("zh-CN", "尖刺护体")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Downpour"), ("zh-CN", "战斗时终极技能充能速度 千针雨")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Downpour"), ("zh-CN", "终极技能自动充能速度 千针雨")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Downpour"), ("zh-CN", "终极技能充能速度（千针雨）")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Downpour"), ("zh-CN", "终极技能（千针雨）")],
    },
    GeneratedHeroSettingName {
        hero: "hazard",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "primaryFireOrbTurnRate%",
        locales: &[("en-US", "Water Staff Orb Turn Rate")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "secondaryFireCost%",
        locales: &[("en-US", "Restorative Stream Drain Rate"), ("zh-CN", "养神泉消耗速度")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "ability1Duration%",
        locales: &[("en-US", "Rushing Torrent Duration Scalar"), ("zh-CN", "飞流步持续时间倍率")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "ability2Kb%",
        locales: &[("en-US", "Guardian Wave Knockback Scalar"), ("zh-CN", "翻江浪击退倍率")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Rushing Torrent Cooldown Time"), ("zh-CN", "飞流步冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Guardian Wave Cooldown Time"), ("zh-CN", "翻江浪冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "enableAbility1",
        locales: &[("en-US", "Rushing Torrent"), ("zh-CN", "飞流步")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "enableAbility2",
        locales: &[("en-US", "Guardian Wave"), ("zh-CN", "翻江浪")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Restorative Stream Cooldown Time"), ("zh-CN", "养神泉冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Restorative Stream Energy Charge Rate"), ("zh-CN", "充能速度 养神泉")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Restorative Stream Maximum Time"), ("zh-CN", "养神泉最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Restorative Stream Recharge Rate"), ("zh-CN", "养神泉充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Restorative Stream"), ("zh-CN", "养神泉")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Tidal Blast"), ("zh-CN", "战斗时终极技能充能速度 惊涛破")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Tidal Blast"), ("zh-CN", "终极技能自动充能速度 惊涛破")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Tidal Blast"), ("zh-CN", "终极技能充能速度（惊涛破）")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Tidal Blast"), ("zh-CN", "终极技能（惊涛破）")],
    },
    GeneratedHeroSettingName {
        hero: "wuyang",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Warding Stance Regen Scalar"), ("zh-CN", "招架姿态恢复倍率")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "ability1Distance%",
        locales: &[("en-US", "Whirlwind Dash Distance"), ("zh-CN", "旋风疾步距离")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "ability2Distance%",
        locales: &[("en-US", "Soaring Slice Distance"), ("zh-CN", "飞空斩击距离")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "ability1Cooldown%",
        locales: &[("en-US", "Whirlwind Dash Cooldown Time"), ("zh-CN", "旋风疾步冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "ability2Cooldown%",
        locales: &[("en-US", "Soaring Slice Cooldown Time"), ("zh-CN", "飞空斩击冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "ability3Cooldown%",
        locales: &[("en-US", " Cooldown Time")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "enableAbility1",
        locales: &[("en-US", "Whirlwind Dash"), ("zh-CN", "旋风疾步")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "enableAbility2",
        locales: &[("en-US", "Soaring Slice"), ("zh-CN", "飞空斩击")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "enableAbility3",
        locales: &[("en-US", "")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "secondaryFireCooldown%",
        locales: &[("en-US", "Warding Stance Cooldown Time"), ("zh-CN", "招架姿态冷却时间")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "secondaryFireEnergyChargeRate%",
        locales: &[("en-US", "Warding Stance Energy Charge Rate"), ("zh-CN", "充能速度 招架姿态")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "secondaryFireMaximumTime%",
        locales: &[("en-US", "Warding Stance Maximum Time"), ("zh-CN", "招架姿态最长时间")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "secondaryFireRechargeRate%",
        locales: &[("en-US", "Warding Stance Recharge Rate"), ("zh-CN", "招架姿态充能速度")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "enableSecondaryFire",
        locales: &[("en-US", "Warding Stance"), ("zh-CN", "招架姿态")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "combatUltGen%",
        locales: &[("en-US", "Ultimate Generation - Combat Sundering Blade"), ("zh-CN", "战斗时终极技能充能速度 斩地巨剑")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "passiveUltGen%",
        locales: &[("en-US", "Ultimate Generation - Passive Sundering Blade"), ("zh-CN", "终极技能自动充能速度 斩地巨剑")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "ultGen%",
        locales: &[("en-US", "Ultimate Generation Sundering Blade"), ("zh-CN", "终极技能充能速度（斩地巨剑）")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "enableUlt",
        locales: &[("en-US", "Ultimate Ability Sundering Blade"), ("zh-CN", "终极技能（斩地巨剑）")],
    },
    GeneratedHeroSettingName {
        hero: "vendetta",
        key: "enablePassive",
        locales: &[("en-US", "")],
    },
];
