#![windows_subsystem = "windows"]

use anyhow::Result;
use eframe::egui;
use egui::{Color32, Frame, Layout, RichText, TextureHandle};
use global_hotkey::{
    GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use windows::{
    Win32::Foundation::HWND,
    Win32::Graphics::Dwm::{
        DWMNCRP_DISABLED, DWMWA_NCRENDERING_POLICY, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
        DwmSetWindowAttribute,
    },
};
mod data;
mod game_info;
mod live_client;
use game_info::GameInfo;
use live_client::LoLLiveClient;

#[cfg(feature = "res_1920")]
const SCREEN_WIDTH: f32 = 1920.0;

#[cfg(feature = "res_2560")]
const SCREEN_WIDTH: f32 = 2560.0;

#[cfg(target_os = "windows")]
fn set_rounded_corners(hwnd: HWND) {
    unsafe {
        let preference = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}
#[cfg(target_os = "windows")]
fn disable_dwm_shadow(hwnd: HWND) {
    unsafe {
        let value: i32 = DWMNCRP_DISABLED.0;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_NCRENDERING_POLICY,
            &value as *const _ as _,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

struct OverlayApp {
    game_info: Arc<Mutex<Option<GameInfo>>>,
    game_data_receiver: mpsc::UnboundedReceiver<GameInfo>,
    icons: HashMap<String, TextureHandle>,
    styles_initialized: bool,
    visible: Arc<Mutex<bool>>,
    hotkey_receiver: mpsc::UnboundedReceiver<()>,
    data_changed: bool,
}

impl OverlayApp {
    fn new(
        game_data_receiver: mpsc::UnboundedReceiver<GameInfo>,
        hotkey_receiver: mpsc::UnboundedReceiver<()>,
    ) -> Self {
        Self {
            game_info: Arc::new(Mutex::new(None)),
            game_data_receiver,
            icons: HashMap::new(),
            styles_initialized: false,
            visible: Arc::new(Mutex::new(true)),
            hotkey_receiver,
            data_changed: true,
        }
    }

    fn get_or_load_icon(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        fallback_emoji: &str,
    ) -> String {
        // For now, return emoji fallback - you can extend this to load actual images
        match name {
            "attack_damage" => "⚔".to_string(),
            "ability_power" => "💫".to_string(),
            "armor" => "🛡".to_string(),
            "magic_resist" => "🟣".to_string(),
            "attack_speed" => "⚡".to_string(),
            "health_regen" => "💉".to_string(),
            "life_steal" => "❤".to_string(),
            "gold" => "💰".to_string(),
            _ => fallback_emoji.to_string(),
        }
    }
}

impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));
        // Initialize text styles once
        if !self.styles_initialized {
            ctx.style_mut(|style| {
                style.text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::new(15.0, egui::FontFamily::Monospace),
                );
                style.text_styles.insert(
                    egui::TextStyle::Heading,
                    egui::FontId::new(20.0, egui::FontFamily::Monospace),
                );
            });
            self.styles_initialized = true;
        }

        // Check for hotkey toggle events
        let mut hotkey_pressed = false;
        while self.hotkey_receiver.try_recv().is_ok() {
            hotkey_pressed = true;
            if let Ok(mut visible) = self.visible.lock() {
                *visible = !*visible;
                if *visible {
                    // Show the window by moving it back to its normal position
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                        SCREEN_WIDTH - 360.0 * 1.3,
                        55.0,
                    )));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        355.0 * 1.3,
                        195.0 * 1.2,
                    )));
                } else {
                    // Hide the window by moving it offscreen and making it tiny
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                        -10000.0, -10000.0,
                    )));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1.0, 1.0)));
                }
                println!("Toggling overlay visibility to: {}", *visible);
            }
        }

        // Get current visibility state
        let is_visible = if let Ok(visible) = self.visible.lock() {
            *visible
        } else {
            true
        };

        // Only request repaint when data changes or hotkey is pressed (better performance)
        if self.data_changed || hotkey_pressed {
            ctx.request_repaint();
            self.data_changed = false;
        } else if is_visible {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        } else {
            // When hidden, repaint much less frequently to save CPU
            ctx.request_repaint_after(std::time::Duration::from_millis(1000));
        }

        // Try to receive new game data without blocking
        while let Ok(new_game_info) = self.game_data_receiver.try_recv() {
            if let Ok(mut game_info) = self.game_info.lock() {
                *game_info = Some(new_game_info);
                self.data_changed = true;
            }
        }

        //ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(true));

        // Only show UI content when visible, but keep the window running
        if is_visible {
            egui::CentralPanel::default()
                .frame(Frame::default().fill(Color32::TRANSPARENT)) // No frame on the central panel itself (on the one right inside it so i have borderrr)
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
                                        let cs_per_min = default_info.scores.creep_score as f64
                                            / game_time_minutes;
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
                                        .heading()
                                        .strong(),
                                    );
                                    });

                                    ui.separator();
                                    ui.add_space(2.0);

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
                                                    for (label, icon, value, icon_color) in [
                                                        (
                                                            "Attack Damage:",
                                                            "⚔",
                                                            format!("{:.0}", stats.attack_damage),
                                                            Color32::from_rgb(255, 100, 100),
                                                        ),
                                                        (
                                                            "Ability Power:",
                                                            "✨",
                                                            format!("{:.0}", stats.ability_power),
                                                            Color32::from_rgb(100, 150, 255),
                                                        ),
                                                        (
                                                            "Armor:",
                                                            "🛡",
                                                            format!("{:.0}", stats.armor),
                                                            Color32::from_rgb(200, 200, 100),
                                                        ),
                                                        (
                                                            "Magic Resist:",
                                                            "🔮",
                                                            format!("{:.0}", stats.magic_resist),
                                                            Color32::from_rgb(150, 100, 255),
                                                        ),
                                                        (
                                                            "CS/min:",
                                                            "🗡",
                                                            format!("{:.1}", cs_per_min),
                                                            Color32::from_rgb(255, 215, 0),
                                                        ),
                                                        (
                                                            "Move Speed: ",
                                                            "💨",
                                                            format!("{:.0}", stats.move_speed),
                                                            Color32::from_rgb(100, 255, 100),
                                                        ),
                                                        (
                                                            "Crit Chance:",
                                                            "💥",
                                                            format!("{:.0}%", stats.crit_chance),
                                                            Color32::from_rgb(255, 165, 0),
                                                        ),
                                                    ] {
                                                        ui.horizontal(|ui| {
                                                            ui.colored_label(
                                                                Color32::MAGENTA,
                                                                RichText::new(label).strong(),
                                                            );

                                                            ui.with_layout(
                                                                Layout::right_to_left(
                                                                    egui::Align::Center,
                                                                ),
                                                                |ui| {
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(
                                                                            RichText::new(value)
                                                                                .color(
                                                                                    Color32::WHITE,
                                                                                )
                                                                                .strong(),
                                                                        );
                                                                        ui.label(
                                                                            RichText::new(icon)
                                                                                .color(icon_color)
                                                                                .size(16.0)
                                                                                .strong(),
                                                                        );
                                                                    });
                                                                },
                                                            );
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
                                                    for (label, icon, value, icon_color) in [
                                                        (
                                                            "Lethality:",
                                                            "",
                                                            if stats.physical_lethality== 0.0 && stats.armor_pen == 1.0 {
                                                                "LDR Zaddy?".to_string()
                                                            } else if stats.lethality == 0.0 && stats.armor_pen == 1.0 {
                                                                "LDR Zaddy?".to_string() // eventually make this item to buy. to_strin()?
                                                            } else {
                                                                format!(
                                                                    "{:.0} | %{:.2}",
                                                                    stats.lethality,
                                                                    100.0 * (1.0 - stats.armor_pen)
                                                                )
                                                            },
                                                            Color32::from_rgb(255, 80, 80),
                                                        ),
                                                        (
                                                            "Mag Pen:",
                                                            "",
                                                            if (stats.magic_lethality + stats.magic_pen) == 0.0 && stats.magic_pen_percent == 1.0 {
                                                                "Void Stiffy?".to_string()
                                                            } else {
                                                                format!(
                                                                    "{:.0} | %{:.2}",
                                                                    stats.magic_lethality
                                                                        + stats.magic_pen,
                                                                    100.0
                                                                        * (1.0
                                                                            - stats.magic_pen_percent)
                                                                )
                                                            },
                                                            Color32::PURPLE,
                                                        ),
                                                        (
                                                            "Att. Sp.:",
                                                            "",
                                                            format!(
                                                                "{:.2} atk/s",
                                                                stats.attack_speed
                                                            ),
                                                            Color32::from_rgb(255, 255, 100),
                                                        ),
                                                        (
                                                            "HP Regen:",
                                                            "",
                                                            format!(
                                                                "{:.1} hp/s",
                                                                stats.health_regen_rate
                                                            ),
                                                            Color32::from_rgb(100, 255, 100),
                                                        ),
                                                        (
                                                            "Life Steal:",
                                                            "❤",
                                                            format!("{:.0}%", stats.life_steal),
                                                            Color32::from_rgb(255, 100, 100),
                                                        ),
                                                        (
                                                            "Total Gold:",
                                                            "💰",
                                                            format!("{:.0}", total_gold),
                                                            Color32::from_rgb(255, 215, 0),
                                                        ),
                                                    ] {
                                                        ui.horizontal(|ui| {
                                                            ui.colored_label(
                                                                Color32::MAGENTA,
                                                                RichText::new(label).strong(),
                                                            );

                                                            ui.with_layout(
                                                                Layout::right_to_left(
                                                                    egui::Align::Center,
                                                                ),
                                                                |ui| {
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(
                                                                            RichText::new(value)
                                                                                .color(
                                                                                    Color32::WHITE,
                                                                                )
                                                                                .strong(),
                                                                        );
                                                                        ui.label(
                                                                            RichText::new(icon)
                                                                                .color(icon_color)
                                                                                .size(16.0)
                                                                                .strong(),
                                                                        );
                                                                    });
                                                                },
                                                            );
                                                        });
                                                    }

                                                    ui.horizontal(|ui| {
                                                        if is_dead && respawn_timer > 0.0 {
                                                            ui.colored_label(Color32::RED, RichText::new("DEAD:").strong());
                                                            ui.colored_label(
                                                                Color32::YELLOW,
                                                                RichText::new(format!("{:.1}s", respawn_timer)).strong(),
                                                            );
                                                        } else {
                                                            ui.colored_label(
                                                                Color32::GREEN,
                                                                RichText::new("ALIVE").strong(),
                                                            );
                                                        }
                                                    });
                                                },
                                            );
                                        });
                                    });
                                });
                            } else {
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("Waiting for game data...").strong());
                                });
                            }
                        });
                    });
                });
        } // Close the if is_visible block

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

fn setup_global_hotkey(sender: mpsc::UnboundedSender<()>) -> Result<GlobalHotKeyManager> {
    let manager = GlobalHotKeyManager::new()?;

    // Create Ctrl+Shift+X hotkey
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyX);
    manager.register(hotkey)?;

    // Spawn a background thread to listen for hotkey events
    std::thread::spawn(move || {
        use global_hotkey::GlobalHotKeyEvent;

        loop {
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state == HotKeyState::Pressed {
                    let _ = sender.send(());
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    Ok(manager)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting League of Legends Overlay...");

    // Set up global hotkey
    let (hotkey_sender, hotkey_receiver) = mpsc::unbounded_channel::<()>();
    let _hotkey_manager = setup_global_hotkey(hotkey_sender)?;

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
            .with_fullsize_content_view(true)
            .with_transparent(true)
            .with_always_on_top(),
        ..Default::default()
    };

    let app = OverlayApp::new(receiver, hotkey_receiver); // Move receivers once here

    if let Err(e) = eframe::run_native(
        "League Overlay",
        options,
        Box::new(move |cc| {
            #[cfg(target_os = "windows")]
            {
                if let Ok(raw_handle) = cc.window_handle() {
                    if let RawWindowHandle::Win32(win32_handle) = raw_handle.as_raw() {
                        let hwnd = HWND(win32_handle.hwnd.get());
                        set_rounded_corners(hwnd);
                        disable_dwm_shadow(hwnd);
                    }
                }
            }
            Ok(Box::new(app)) // use the already created app here
        }),
    ) {
        println!("Failed to run overlay: {}", e);
    }

    Ok(())
}
