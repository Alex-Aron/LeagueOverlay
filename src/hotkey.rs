use anyhow::Result;
use global_hotkey::{
    GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use tokio::sync::mpsc;

pub fn setup_global_hotkey(sender: mpsc::UnboundedSender<()>) -> Result<GlobalHotKeyManager> {
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
