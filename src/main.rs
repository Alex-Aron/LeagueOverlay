#![windows_subsystem = "windows"]

use anyhow::Result;
use eframe::egui;
use egui::{Color32, Frame, ViewportCommand};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
mod data;
mod game_info;
mod live_client;
use game_info::GameInfo;
use live_client::LoLLiveClient;

#[cfg(feature = "res_1920")]
const SCREEN_WIDTH: f32 = 1920.0;

#[cfg(feature = "res_2560")]
const SCREEN_WIDTH: f32 = 2560.0;

struct OverlayApp {
    game_info: Arc<Mutex<Option<GameInfo>>>,
    game_data_receiver: mpsc::UnboundedReceiver<GameInfo>,
}

impl OverlayApp {
    fn new(game_data_receiver: mpsc::UnboundedReceiver<GameInfo>) -> Self {
        Self {
            game_info: Arc::new(Mutex::new(None)),
            game_data_receiver,
        }
    }
}

impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Try to receive new game data without blocking
        while let Ok(new_game_info) = self.game_data_receiver.try_recv() {
            if let Ok(mut game_info) = self.game_info.lock() {
                *game_info = Some(new_game_info);
            }
        }

        ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(true));

        egui::CentralPanel::default()
            .frame(Frame::NONE) // No frame on the central panel itself (on the one right inside it so i have borderrr)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    Frame {
                        corner_radius: egui::CornerRadius::same(10),
                        fill: egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
                        stroke: egui::Stroke::new(2.0, egui::Color32::WHITE),
                        inner_margin: egui::Margin::same(10),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        // Get display data
                        let display_data = if let Ok(game_info_guard) = self.game_info.lock() {
                            if let Some(ref game_info) = *game_info_guard {
                                let player = &game_info.active_player;
                                let game_time_minutes = game_info.game_data.game_time / 60.0;

                                if let Some(default_info) = game_info
                                    .all_players
                                    .iter()
                                    .find(|p| p.riot_id == player.riot_id)
                                {
                                    let cs_per_min =
                                        default_info.scores.creep_score as f64 / game_time_minutes;
                                    let item_gold: f64 = default_info
                                        .items
                                        .iter()
                                        .map(|item| item.price as f64)
                                        .sum();
                                    let total_gold = player.current_gold + item_gold;

                                    // Copy values needed outside of lock
                                    let is_dead = default_info.is_dead;
                                    let respawn_timer = default_info.respawn_timer;
                                    let stats = player.champion_stats.clone(); // assuming Clone is derived
                                    let riot_id = player.riot_id.clone();

                                    Some((
                                        riot_id,
                                        cs_per_min,
                                        total_gold,
                                        is_dead,
                                        respawn_timer,
                                        stats,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if let Some((
                            riot_id,
                            cs_per_min,
                            total_gold,
                            is_dead,
                            respawn_timer,
                            stats,
                        )) = display_data
                        {
                            // Set font styles once
                            let text_styles = &mut ui.style_mut().text_styles;
                            text_styles.insert(
                                egui::TextStyle::Body,
                                egui::FontId::new(16.0, egui::FontFamily::Proportional),
                            );
                            text_styles.insert(
                                egui::TextStyle::Heading,
                                egui::FontId::new(20.0, egui::FontFamily::Proportional),
                            );

                            ui.vertical(|ui| {
                                ui.vertical_centered(|ui| {
                                    let player_name =
                                        riot_id.split('#').next().unwrap_or("Unknown");
                                    ui.colored_label(
                                        Color32::MAGENTA,
                                        egui::RichText::new(format!(
                                            "Do Not Tilt UwU | {}",
                                            player_name
                                        ))
                                        .heading(),
                                    );
                                });

                                ui.add_space(5.0);
                                ui.separator();
                                ui.add_space(5.0);

                                ui.horizontal_centered(|ui| {
                                    ui.columns(2, |columns| {
                                        // Left column
                                        Frame {
                                            corner_radius: egui::CornerRadius::same(8),
                                            fill: egui::Color32::from_rgba_premultiplied(
                                                0, 0, 0, 50,
                                            ),
                                            stroke: egui::Stroke::new(2.0, egui::Color32::GRAY),
                                            inner_margin: egui::Margin::same(8),
                                            ..Default::default()
                                        }
                                        .show(
                                            &mut columns[0],
                                            |ui| {
                                                for (label, value) in [
                                                    (
                                                        "Attack Damage:",
                                                        format!("{:.0}", stats.attack_damage),
                                                    ),
                                                    (
                                                        "Ability Power:",
                                                        format!("{:.0}", stats.ability_power),
                                                    ),
                                                    ("Armor:", format!("{:.0}", stats.armor)),
                                                    (
                                                        "Magic Resist:",
                                                        format!("{:.0}", stats.magic_resist),
                                                    ),
                                                    ("CS/min:", format!("{:.1}", cs_per_min)),
                                                    (
                                                        "Move Speed:",
                                                        format!("{:.0} m/s", stats.move_speed),
                                                    ),
                                                    (
                                                        "Crit Chance:",
                                                        format!("{:.0}%", stats.crit_chance),
                                                    ),
                                                ] {
                                                    ui.horizontal(|ui| {
                                                        ui.colored_label(Color32::MAGENTA, label);
                                                        ui.label(value);
                                                    });
                                                }
                                            },
                                        );

                                        // Right column
                                        Frame {
                                            corner_radius: egui::CornerRadius::same(8),
                                            fill: egui::Color32::from_rgba_premultiplied(
                                                0, 0, 0, 50,
                                            ),
                                            stroke: egui::Stroke::new(2.0, egui::Color32::GRAY),
                                            inner_margin: egui::Margin::same(8),
                                            ..Default::default()
                                        }
                                        .show(
                                            &mut columns[1],
                                            |ui| {
                                                for (label, value) in [
                                                    (
                                                        "Lethality",
                                                        format!(
                                                            "{:.0} | %{:.2}",
                                                            stats.lethality,
                                                            100.0 * (1.0 - stats.armor_pen)
                                                        ),
                                                    ),
                                                    (
                                                        "Magic Pen",
                                                        format!(
                                                            "{:.0} | %{:.2}",
                                                            stats.magic_lethality + stats.magic_pen,
                                                            100.0 * (1.0 - stats.magic_pen_percent)
                                                        ),
                                                    ),
                                                    (
                                                        "Attack Speed:",
                                                        format!(
                                                            "⚔ {:.2} atk/s",
                                                            stats.attack_speed
                                                        ),
                                                    ),
                                                    (
                                                        "HP Regen:",
                                                        format!(
                                                            "💉{:.1} hp/s",
                                                            stats.health_regen_rate
                                                        ),
                                                    ),
                                                    (
                                                        "Life Steal:",
                                                        format!("❤ {:.0}%", stats.life_steal),
                                                    ),
                                                    ("TotalGold:", format!("💰{:.0}", total_gold)),
                                                ] {
                                                    ui.horizontal(|ui| {
                                                        ui.colored_label(Color32::MAGENTA, label);
                                                        ui.label(value);
                                                    });
                                                }

                                                ui.horizontal(|ui| {
                                                    if is_dead && respawn_timer > 0.0 {
                                                        ui.colored_label(Color32::RED, "DEAD:");
                                                        ui.colored_label(
                                                            Color32::YELLOW,
                                                            format!("{:.1}s", respawn_timer),
                                                        );
                                                    } else {
                                                        ui.colored_label(Color32::GREEN, "ALIVE");
                                                    }
                                                });
                                            },
                                        );
                                    });
                                });
                            });
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.label("Waiting for game data...");
                            });
                        }
                    });
                });
            });

        ctx.request_repaint();
    }
}

async fn game_data_fetcher(sender: mpsc::UnboundedSender<GameInfo>) -> Result<()> {
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

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting League of Legends Overlay...");
    // Create mpsc channel for game data
    let (sender, receiver) = mpsc::unbounded_channel::<GameInfo>();

    // Spawn background task to fetch game data
    let data_sender = sender.clone();
    tokio::spawn(async move {
        if let Err(e) = game_data_fetcher(data_sender).await {
            println!("Game data fetcher error: {}", e);
        }
    });

    // Set up the GUI
    let vertical_scale: f32 = 1.2;
    let horizontal_scale: f32 = 1.3;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([355.0 * horizontal_scale, 195.0 * vertical_scale])
            .with_position(egui::pos2(SCREEN_WIDTH - 360.0 * horizontal_scale, 55.0))
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(),
        ..Default::default()
    };

    let app = OverlayApp::new(receiver);

    if let Err(e) = eframe::run_native("League Overlay", options, Box::new(|_cc| Ok(Box::new(app))))
    {
        println!("Failed to run overlay: {}", e);
    }

    Ok(())
}
