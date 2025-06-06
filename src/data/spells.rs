use ::serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpells {
    pub summoner_spell_one: SummonerSpell,
    pub summoner_spell_two: SummonerSpell,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpell {
    pub display_name: String,
    pub raw_description: String,
    pub raw_display_name: String,
}
