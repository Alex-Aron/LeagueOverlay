use crate::GameInfo;
use eframe::egui;
use egui::{Color32, Frame, Layout, RichText, TextureHandle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[cfg(feature = "res_1920")]
const SCREEN_WIDTH: f32 = 1920.0;

#[cfg(feature = "res_2560")]
const SCREEN_WIDTH: f32 = 2560.0;

pub struct OverlayApp {
    game_info: Arc<Mutex<Option<GameInfo>>>,
    game_data_receiver: mpsc::UnboundedReceiver<GameInfo>,
    icons: HashMap<String, TextureHandle>,
    styles_initialized: bool,
    visible: Arc<Mutex<bool>>,
    hotkey_receiver: mpsc::UnboundedReceiver<()>,
    data_changed: bool,
}

impl OverlayApp {
    pub fn new(
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
        _ctx: &egui::Context,
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
        // Initialize text styles once
        if !self.styles_initialized {
            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));

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

        // Only show UI content when visible, but keep the window running
        if is_visible {
            self.render_ui(ctx);
        }

        ctx.request_repaint();
    }
}

impl OverlayApp {
    fn render_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::default().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    Frame {
                        corner_radius: egui::CornerRadius::same(10),
                        fill: egui::Color32::TRANSPARENT,
                        stroke: egui::Stroke::new(
                            2.0,
                            egui::Color32::from_rgba_premultiplied(255, 255, 255, 150),
                        ),
                        inner_margin: egui::Margin::same(10),

                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        self.render_game_stats(ui);
                    });
                });
            });
    }

    fn render_game_stats(&mut self, ui: &mut egui::Ui) {
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
                    let cs_per_min = default_info.scores.creep_score as f64 / game_time_minutes;
                    let item_gold: f64 = default_info
                        .items
                        .iter()
                        .map(|item| item.price as f64)
                        .sum();
                    let total_gold = player.current_gold + item_gold;

                    Some((
                        player.riot_id.clone(),
                        cs_per_min,
                        total_gold,
                        default_info.is_dead,
                        default_info.respawn_timer as f64,
                        player.champion_stats.clone(),
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

        if let Some((riot_id, cs_per_min, total_gold, is_dead, respawn_timer, stats)) = display_data
        {
            ui.vertical(|ui| {
                ui.vertical_centered(|ui| {
                    let player_name = riot_id.split('#').next().unwrap_or("Unknown");
                    ui.colored_label(
                        Color32::MAGENTA,
                        egui::RichText::new(format!("Do Not Tilt UwU | {}", player_name))
                            .heading()
                            .strong(),
                    );
                });

                ui.separator();
                ui.add_space(2.0);

                ui.horizontal_centered(|ui| {
                    ui.columns(2, |columns| {
                        self.render_left_column(&mut columns[0], &stats, cs_per_min);
                        self.render_right_column(
                            &mut columns[1],
                            &stats,
                            total_gold,
                            is_dead,
                            respawn_timer,
                        );
                    });
                });
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Waiting for game data...").strong());
            });
        }
    }

    fn render_left_column(
        &self,
        ui: &mut egui::Ui,
        stats: &crate::data::players::ChampionStats,
        cs_per_min: f64,
    ) {
        Frame {
            corner_radius: egui::CornerRadius::same(8),
            fill: egui::Color32::from_rgba_premultiplied(0, 0, 0, 80),
            stroke: egui::Stroke::new(
                2.0,
                egui::Color32::from_rgba_premultiplied(128, 128, 128, 150),
            ),
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| {
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
                self.render_stat_row(ui, label, icon, &value, icon_color);
            }
        });
    }

    fn render_right_column(
        &self,
        ui: &mut egui::Ui,
        stats: &crate::data::players::ChampionStats,
        total_gold: f64,
        is_dead: bool,
        respawn_timer: f64,
    ) {
        Frame {
            corner_radius: egui::CornerRadius::same(8),
            fill: egui::Color32::from_rgba_premultiplied(0, 0, 0, 80),
            stroke: egui::Stroke::new(
                2.0,
                egui::Color32::from_rgba_premultiplied(128, 128, 128, 150),
            ),
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        }
        .show(ui, |ui| {
            for (label, icon, value, icon_color) in [
                (
                    "Lethality:",
                    "",
                    if (stats.lethality == 0.0 || stats.physical_lethality == 0.0)
                        && stats.armor_pen == 1.0
                    {
                        "Farm!!!".to_string()
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
                    if (stats.magic_lethality + stats.magic_pen) == 0.0
                        && stats.magic_pen_percent == 1.0
                    {
                        "Keep it up!".to_string()
                    } else {
                        format!(
                            "{:.0} | %{:.2}",
                            stats.magic_lethality + stats.magic_pen,
                            100.0 * (1.0 - stats.magic_pen_percent)
                        )
                    },
                    Color32::PURPLE,
                ),
                (
                    "Att. Sp.:",
                    "",
                    format!("{:.2} atk/s", stats.attack_speed),
                    Color32::from_rgb(255, 255, 100),
                ),
                (
                    "HP Regen:",
                    "",
                    format!("{:.1} hp/s", stats.health_regen_rate),
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
                self.render_stat_row(ui, label, icon, &value, icon_color);
            }

            ui.horizontal(|ui| {
                if is_dead && respawn_timer > 0.0 {
                    ui.colored_label(Color32::RED, RichText::new("DEAD:").strong());
                    ui.colored_label(
                        Color32::GRAY,
                        RichText::new(format!("{:.1}s", respawn_timer)).strong(),
                    );
                } else {
                    ui.colored_label(Color32::GREEN, RichText::new("ALIVE").strong());
                }
            });
        });
    }

    fn render_stat_row(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        icon: &str,
        value: &str,
        icon_color: Color32,
    ) {
        ui.horizontal(|ui| {
            ui.colored_label(Color32::MAGENTA, RichText::new(label).strong());

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(value).color(Color32::GRAY).strong());
                    ui.label(RichText::new(icon).color(icon_color).size(16.0).strong());
                });
            });
        });
    }
}
