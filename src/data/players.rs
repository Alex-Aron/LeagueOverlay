use crate::data::items::Item;
use crate::data::runes::Rune;
use crate::data::spells::SummonerSpells;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Abilities {
    pub passive: AbilityInfo,
    pub q: AbilityInfo,
    pub w: AbilityInfo,
    pub e: AbilityInfo,
    pub r: AbilityInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityInfo {
    pub ability_level: u8,
    pub display_name: String,
    pub id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub assists: u16,
    pub deaths: u16,
    pub kills: u16,
    pub creep_score: u16,
    pub ward_score: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionStats {
    pub ability_haste: f64,
    pub ability_power: f64,
    pub armor: f64,
    #[serde(rename = "armorPenetrationFlat")]
    pub lethality: f64,
    #[serde(rename = "armorPenetrationPercent")]
    pub armor_pen: f64,
    pub attack_damage: f64,
    pub attack_range: f64,
    pub attack_speed: f64,
    #[serde(rename = "bonusArmorPenetrationPercent")]
    pub bonus_armor_pen: f64,
    #[serde(rename = "bonusMagicPenetrationPercent")]
    pub bonus_magic_pen: f64,
    pub crit_chance: f64,
    pub crit_damage: f64,
    pub current_health: f64,
    pub heal_shield_power: f64,
    pub health_regen_rate: f64,
    pub life_steal: f64,
    pub magic_lethality: f64,
    #[serde(rename = "magicPenetrationFlat")]
    pub magic_pen: f64,
    #[serde(rename = "magicPenetrationPercent")]
    pub magic_pen_percent: f64,
    pub magic_resist: f64,
    pub max_health: f64,
    pub move_speed: f64,
    pub omnivamp: f64,
    pub physical_lethality: f64,
    pub physical_vamp: f64,
    pub resource_max: f64,
    pub resource_regen_rate: f64,
    pub resource_type: String,
    pub resource_value: f64,
    pub spell_vamp: f64,
    pub tenacity: f64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActivePlayer {
    pub riot_id: String,
    pub champion_stats: ChampionStats,
    pub level: u8,
    pub team_relative_colors: bool,
    pub current_gold: f64,
    pub player_op: Option<Player>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub champion_name: String,
    pub is_bot: bool,
    pub is_dead: bool,
    pub level: u8,
    pub position: String,
    pub respawn_timer: f32,
    pub riot_id: String,
    pub team: String,
    pub items: Vec<Item>,
    pub runes: Rune,
    pub scores: Score,
    #[serde(rename = "summonerSpells")]
    pub spells: SummonerSpells,
    #[serde(skip)]
    pub abilities: Option<Abilities>,
}
