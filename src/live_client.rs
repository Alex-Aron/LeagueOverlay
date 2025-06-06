use crate::game_info::{GameData};
use anyhow::Result;
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use std::time::Duration;
/*
This code is for the Leage Live Game API. This is in game stats aside from the normal API.
This will be combined iwth dragontail data to provide ability information as well
Item information wont be implemented because you can look up items in game (in due timeeee)
*/

pub struct LoLLiveClient {
    client: Client,
    base_url: String,
}

impl LoLLiveClient {
    pub fn new() -> Result<Self> {
        //Create a client that accepts invalid certificates cuz it error'd (local???)
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(5))
            .build()?;

        let base_url = "https://127.0.0.1:2999/liveclientdata".to_string();

        Ok(LoLLiveClient { client, base_url })
    }

    async fn make_request<T>(&self, endpoint: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        //Get raw text first for better error reporting
        let text = response.text().await?;

        // Try to parse with detailed error info
        match serde_json::from_str::<T>(&text) {
            Ok(data) => Ok(data),
            Err(e) => {
                println!("=== PARSING ERROR ===");
                println!("Endpoint: {}", endpoint);
                println!("Error: {}", e);
                println!("Error location: line {}, column {}", e.line(), e.column());

                // Show the problematic area of JSON
                let lines: Vec<&str> = text.lines().collect();
                let error_line = e.line().saturating_sub(1);
                let start = error_line.saturating_sub(2);
                let end = (error_line + 3).min(lines.len());

                println!("Context around error:");
                for (i, line) in lines[start..end].iter().enumerate() {
                    let line_num = start + i + 1;
                    let marker = if line_num == e.line() { ">>> " } else { "    " };
                    println!("{}{}: {}", marker, line_num, line);
                }
                println!("=== END PARSING ERROR ===");

                Err(anyhow::anyhow!("JSON parsing failed: {}", e))
            }
        }
    }

    pub async fn is_game_active(&self) -> bool {
        self.make_request::<GameData>("/gamestats").await.is_ok()
    }

    pub async fn get_all_game_data(&self) -> Result<serde_json::Value> {
        self.make_request("/allgamedata").await
    }

    #[allow(dead_code)]
    pub async fn get_events(&self) -> Result<serde_json::Value> {
        self.make_request("/eventdata").await
    }

    #[allow(dead_code)]
    pub async fn get_player_scores(&self) -> Result<serde_json::Value> {
        self.make_request("/playerscores").await
    }
}
