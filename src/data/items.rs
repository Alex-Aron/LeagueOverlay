use ::serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(rename = "displayName")]
    pub name: String,
    pub can_use: bool,
    pub slot: u8,
    pub count: u8,
    pub price: u32,
    #[serde(rename = "itemID")]
    pub id: u32,
}
