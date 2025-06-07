#![windows_subsystem = "windows"]
use anyhow::Result;
use eframe::egui;
use tokio::sync::mpsc;
mod data;
mod fetcher;
mod game_info;
mod hotkey;
mod live_client;
mod overlay;

#[cfg(target_os = "windows")]
mod windows;

use fetcher::game_data_fetcher;
use game_info::GameInfo;
use hotkey::setup_global_hotkey;
use overlay::OverlayApp;

#[cfg(target_os = "windows")]
use windows::apply_window_styling;

#[cfg(feature = "res_1920")]
const SCREEN_WIDTH: f32 = 1920.0;
#[cfg(feature = "res_1920")]
const SCREEN_HEIGHT: f32 = 1080.0;

#[cfg(feature = "res_2560")]
const SCREEN_WIDTH: f32 = 2560.0;
#[cfg(feature = "res_2560")]
const SCREEN_HEIGHT: f32 = 1440.0;

// Embed the icon PNG file at compile time
const ICON_PNG_BYTES: &[u8] = include_bytes!("assets/icon.png");

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
            .with_icon({
                let mut decoder = png::Decoder::new(ICON_PNG_BYTES);
                decoder.set_transformations(png::Transformations::ALPHA);
                let mut reader = decoder.read_info().unwrap();
                let mut buf = vec![0; reader.output_buffer_size()];
                let info = reader.next_frame(&mut buf).unwrap();
                egui::IconData {
                    rgba: buf,
                    width: info.width,
                    height: info.height,
                }
            })
            .with_fullsize_content_view(true)
            .with_transparent(true)
            .with_always_on_top(),
        ..Default::default()
    };

    let app = OverlayApp::new(receiver, hotkey_receiver);

    if let Err(e) = eframe::run_native(
        "League Overlay",
        options,
        Box::new(move |cc| {
            #[cfg(target_os = "windows")]
            apply_window_styling(cc);

            Ok(Box::new(app))
        }),
    ) {
        println!("Failed to run overlay: {}", e);
    }

    Ok(())
}
