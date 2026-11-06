# pyCTF - Rust Port

A Rust port of the Python/Pygame Capture the Flag game, built with **Macroquad**.

## Project Status

**In progress.** The core game (physics, collision, scoring, menus) is ported and compiles. Online multiplayer is not yet fully implemented.

## Quick Start

```bash
cd pyctf-rust
cargo run
```

### Dependencies
- Rust (stable, edition 2024)
- Uses **Macroquad** (0.4) for the game window, rendering, and input.

## Project Structure

```
pyctf-rust/
  src/
    main.rs      - Entry point, game loop, online state stubs
    game.rs      - Entities, physics, collision, scoring
    assets/      - Font, background, flags (copied from original)
  docs/          - pygodide build output (web version)
  elements/      - Original Python game assets (png, ogg)
```

## Game Configuration

Key mappings (matching the original Python game):

| Player | Movement | Jump | Fall | Dash Left | Dash Right |
|--------|----------|------|------|-----------|------------|
| 1 (Red) | WASD | W | S | Q | E |
| 2 (Blue) | IJKL | I | K | U | O |

## What's Implemented

- Player movement (WASD, dash Q/E, dash U/O) - verified
- Platform physics (jump, fall, dash)
- Collision detection (platforms, void, flags)
- Scoring and respawn
- Game menus (keybinds, mode selection placeholder)
- Sound effects (jump, woosh)

## What's Left

- [ ] Online multiplayer (host/guest) - requires WebRTC relay
- [ ] Match timer and win conditions
- [ ] Full menu system with mode selection
- [ ] Music playback (gamemusic.ogg)

## Built With

- **Rust** (stable)
- **Macroquad** (2D game framework)
- **Cargo** (package manager)

## Credits

- Original Python version: Nicolas Campan & Matthew Duff
- Music: Nicolas Campan & John Brennan
