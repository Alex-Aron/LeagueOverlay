use ::serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RuneType {
    #[serde(rename = "displayName")]
    pub name: String,
    pub id: u32,
    #[serde(rename = "rawDescription")]
    pub description: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Rune {
    pub keystone: RuneType,
    pub primary_rune_tree: RuneType,
    pub secondary_rune_tree: RuneType,
}
