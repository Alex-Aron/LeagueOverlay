# League of Legends Overlay
A real-time overlay application for League of Legends that displays your champion's stats and game information in a sleek, transparent window.

## Features

- **Real-time Stats Display**: Shows live champion statistics including Attack Damage, Ability Power, Armor, Magic Resist, and more
- **Transparent Overlay**: Non-intrusive overlay that sits on top of your game
- **Player Information**: Displays your summoner name and personalized "Do Not Tilt" message
- **Two-Column Layout**: Organized display with basic stats on the left and advanced stats on the right
- **Death Timer**: Shows respawn countdown when dead
- **CS/Min Tracking**: Real-time creep score per minute calculation
- **Total Gold**: Combines current gold with item values

## Stuff I Want To Add
- Popup with mini descriptions of enemy champs
- Menu
- Configuration Settings
- Events Popup: If a recent event was a multikill and the riotId is ActivePlayer, I want to do a yippee popup.
- Show summ spells
- Have my own rune descriptions (short and probably intentionally odd)
- Make main.rs less clunky

## Stats Displayed

### Left Column
- Attack Damage
- Ability Power  
- Armor
- Magic Resist
- CS/min
- Move Speed
- Crit Chance

### Right Column
- Lethality (flat + % armor penetration)
- Magic Penetration (flat + %)
- Attack Speed ⚔
- HP Regeneration 💉
- Life Steal ❤
- Total Gold 💰
- Alive/Dead Status

## Requirements

- **Rust** (latest stable version) -> If you just give someone exe they can use w/o.
- **League of Legends** client running
- **Windows** (tested on Windows 10/11)
- Screen resolution support for 1920x1080 or 2560x1440

## Installation & Usage

1. **Clone the repository**
   ```bash
   git clone https://github.com/Alex-Aron/LeagueOverlay
   cd league_overlay
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the overlay**
   ```bash
   # For 1920x1080 resolution
   cargo run --release --features res_1920
   
   # For 2560x1440 resolution  
   cargo run --release --features res_2560
   ```

4. **Start a League of Legends game** - the overlay will automatically detect when you're in-game and start displaying stats

## Configuration

The overlay is configured for different screen resolutions using Cargo features:

- `--features res_1920`: For 1920x1080 displays
- `--features res_2560`: For 2560x1440 displays

The overlay automatically positions itself in the top-right corner of your screen.

**Note**: This overlay is designed for personal use and learning purposes. Make sure to comply with Riot Games' terms of service when using any third-party applications with League of Legends.
