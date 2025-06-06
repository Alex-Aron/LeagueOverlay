use crate::data::events::EventsWrapper;
use crate::data::players::{ActivePlayer, Player};
use ::serde::Deserialize;

/*
This will hold information about the game
Starting with live client information only
It will be updated with more information as needed
*/
///aatrox            "resourceType": "BLOODWELL",
///yone WIND  

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game_mode: String,
    pub game_time: f64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub active_player: ActivePlayer,
    pub all_players: Vec<Player>,
    pub events: EventsWrapper,
    pub game_data: GameData,
}
