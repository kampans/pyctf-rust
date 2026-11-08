# pyCTF - Rust/Macroquad Port (pyrust)

**pyCTF** is the original Python/pygame version of the Capture the Flag game.
**pyrust** is the new Rust/Macroquad port of this version. 

## Gameplay

- Choose **CAPTURE THE FLAG** or **COIN RACE** in the lobby by clicking the mode selector or pressing **F3**. Press **Escape** during play to return and switch modes; starting resets scores.
- In **Coin Race**, both players race for an animated coin on a random upper platform. Players start on safe platforms at equal straight-line distances from the coin (at least 400 pixels away), with at least 400 pixels between players. Player sides are randomized each race.
- A coin pickup awards 10 points and starts a new race. Simultaneous pickups restart without points. Contact between players does not score; falling awards the opponent 5 points and returns the fallen player to their race spawn. The NPC targets the coin in this mode.
- Coin Race ends after 120 seconds with a translucent match summary showing both scores and the winner (or a draw). Click **Play Again** / press **Enter** for a fresh match with scores reset, or click **Lobby** / press **Escape** to change settings. Gameplay freezes behind the summary while the regular rendering and input loop continues, without blocking waits.
- Two players (Red vs Blue) compete to capture the flag
- P1 is always human; choose **P2: HUMAN** or **P2: NPC** in the start menu (click the toggle or press **F2**). Human is the default.
- The NPC navigates platforms, jumps, drops and dashes using the same physics as a human. It pursues the active flag as attacker and chases P1 as defender, adapting when roles swap.
- NPC mode ignores P2 keyboard controls and marks the blue player with `[NPC]`. Names remain editable in either mode.
- The attacker tries to grab the opponent's flag and return it to their base
- The defender tries to stop the attacker or catch them
- Rounds last 120 seconds; scoring is based on flag captures, defender catches, and void kills
- Players can customize their names (up to 12 characters)

## Controls

- **Music controls (top right, in the lobby and match summary)**: click **Mute Music / Unmute Music**, or click and drag the volume slider. Controls are hidden and inactive during gameplay. Music starts at 35% and loops, automatically on desktop and after your first click or keypress in browser builds. Muting preserves your chosen volume and leaves jump/dash sound effects enabled. Settings carry across rematches and lobby visits during the session.

### Player 1 (Red)
- **Move**: A / D
- **Jump**: W
- **Fall/Hang**: S
- **Dash left**: Q
- **Dash right**: E

### Player 2 (Blue)
- **Move**: J / L
- **Jump**: I
- **Fall/Hang**: K
- **Dash left**: U
- **Dash right**: O

### General
- **Enter/Space**: Start game (from lobby)
- **Tab**: Switch between name input fields
- **F2**: Toggle P2 between human and NPC in the lobby
- **F3**: Switch between Capture the Flag and Coin Race in the lobby
- **Escape**: Return to the start menu; starting again resets the match
- **F11**: Toggle fullscreen (Macroquad default)
- **9**: Toggle hitbox display (debug)

## Installation

### Prerequisites
- Rust and Cargo installed (via `rustup`)
- Macroquad game framework

### Building
```bash
git clone <https://github.com/kampans/pyctf-rust>
cd pyctf-rust
cargo run
```

The game will launch in a 1600x900 window.

## Project Structure

```
pyctf-rust/
├── assets/
│   ├── pictures/
│   │   ├── pyctf.png          # Original Python version screenshot
│   │   └── pyrust.png         # New Rust version screenshot
│   ├── background.png         # Game background
│   ├── blueflag.png           # Blue team flag
│   ├── gamemusic.ogg          # Background music
│   ├── jump.ogg               # Jump sound effect
│   ├── player_blue.png        # Blue player sprite
│   ├── player_red.png         # Red player sprite
│   ├── redflag.png            # Red team flag
│   ├── Untitled.png           # Placeholder texture
│   └── woosh.ogg              # Dash/swoosh sound effect
├── src/
│   ├── main.rs                # Entry point - window config, game state, lobby
│   ├── game.rs                # Game entities, collision, scoring
│   ├── motion.rs              # Shared player and NPC prediction physics
│   └── npc.rs                 # P2 navigation, attack and defense
├── Cargo.toml                 # Rust package configuration
└── .gitignore
```

### Screenshots
- **pyctf.png**: Screenshot of the original Python/pygame version
- **pyrust.png**: Screenshot of this Rust/Macroquad port

<img src="assets/pictures/pyctf.png" alt="pyCTF Original" width="343">
<img src="assets/pictures/pyrust.png" alt="pyrust Rust Port" width="400">

## Game Mechanics

- **Physics**: Tuned for ~60 FPS with acceleration, friction, and gravity
- **Platforming**: Players can stand on platforms and fall through them
- **Flag carrying**: When a player holds the flag, the defender can catch them
- **Scoring**:
  - 10 points for capturing the flag
  - 10 points for defender catching the attacker
  - 5 points for opponent falling into the void
- **Round timer**: 120 seconds per round; roles swap when time runs out
- **Spawn protection**: Players spawn with a flash effect and invulnerability

## Validation

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

The navigation tests check every platform pair and simulate NPC movement from varied spawn positions to targets across the arena, checking that it reaches them without falling into the void.

## Development

Built with:
- **Macroquad 0.4** - Rust game library
- **Rand** - Random number generation

The game aims to faithfully port the original pyCTF gameplay experience to Rust while taking advantage of Macroquad's cross-platform capabilities.

## License

License on this code is yet to be decided.
