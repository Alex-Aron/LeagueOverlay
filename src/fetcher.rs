use crate::{GameInfo, live_client::LoLLiveClient};
use anyhow::Result;
use tokio::sync::mpsc;

pub async fn game_data_fetcher(sender: mpsc::UnboundedSender<GameInfo>) -> Result<()> {
    println!("Starting League of Legends Live Client API connection...");
    let client = LoLLiveClient::new()?;

    loop {
        if !client.is_game_active().await {
            println!("No active game detected. Waiting...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        match client.get_all_game_data().await {
            Ok(data) => {
                // Try to parse the game data
                if let Ok(game_info) = serde_json::from_value::<GameInfo>(data) {
                    if sender.send(game_info).is_err() {
                        println!("Failed to send game data - receiver likely dropped");
                        break;
                    }
                } else {
                    println!("Failed to parse game data structure");
                }
            }
            Err(e) => {
                println!("Error fetching game data: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    Ok(())
}
