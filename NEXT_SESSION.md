# Next session: multiplayer, match history, and polish

## Start here

This file is the handoff for the next development session. These are planned goals, not implemented features. Begin by reading this file and `WEB.md`, inspecting the current code and Git status, and agreeing on the first implementation milestone with Nicolas.

The main next update is **online multiplayer with room codes**, followed by **persistent match results displayed automatically on the website as a battle log and leaderboard**. Preserve the existing local multiplayer and NPC modes.

## What Nicolas wants

- A player can create a multiplayer room and share a code with a friend.
- Their friend can enter the code and join from another device/browser.
- Both players can enter their own names and battle each other.
- Games are logged persistently, with their outcomes automatically shown on the website.
- A battle log and leaderboard show the results of those games.
- Later improvements include replacement sprite sheets, different sprites/characters, more sound effects, and quality-of-life fixes.

## Current project

- Rust/Macroquad game, with native desktop and a working WebAssembly website build.
- Game logic: `src/game.rs`; shared movement: `src/motion.rs`; NPC routing: `src/npc.rs`.
- Screens, lobby, input, and fixed-step simulation: `src/main.rs`.
- Browser viewport/lifecycle: `src/platform.rs`; audio: `src/audio.rs`.
- Website source: `web/`; public patch notes: `web/updates.txt`.
- Build output: generated, ignored `dist/`. Edit source files, then rebuild.
- `WEB.md` covers local builds, browser checks, and static hosting.
- No multiplayer backend, database, room service, or persistent leaderboard is implemented yet.
- Cloudflare hosting and private Git integration were discussed. Do not assume an account, deployed project, domain, or deployment connection has been configured; inspect/confirm the actual state first.
- The original Python/pygame DCU / CTYI project is https://github.com/kampans/pyCTF. The website links to it and describes this game as a more polished Rust port.

### Existing rules to preserve

- **CTF:** two 120-second halves; roles reverse exactly once. Scores carry over. After the second half, show the winner/draw and Play Again/Lobby actions.
- **Coin Race:** one 120-second match. Coin pickup awards 10 points and starts another race. Simultaneous pickup gives neither player priority. Touching another player does not score.
- Coin Race spawns players on safe platforms, at equal straight-line distances of at least 400 pixels from the coin and at least 400 pixels apart.
- Void deaths award the opponent 5 points.
- Both modes can use human P2 or an NPC. Names and modes are selectable in the lobby.
- Movement runs at 60 simulation ticks per second; player rendering interpolates between ticks. NPC next routes are precomputed; HUD layouts are cached.
- Music starts muted. Music volume/mute controls appear only in the lobby and match summary. Jump/dash SFX remain enabled.

### Website preferences to preserve

- Game window first, with no site header above it. Current desktop sizing: up to 1360 pixels wide and 84% of viewport height.
- Title and read-only patch notes below the game, followed by the Rust-port background and original repository link.
- No unnecessary marketing taglines, badges, or editable patch-note controls.
- Default dark theme on all devices, with a bottom-corner toggle and an explicit saved light preference respected.
- Use the game's Proggy Clean font and forest-green/red/blue palette. Keep CSS readable, grouped, and formatted with one declaration per line.
- Before activation, the game is greyscale with “Click in the game window to play”. Losing focus returns it to greyscale with “Click in the window to resume”. Clicking anywhere inside restores color and input.
- Current offline focus handling freezes simulation/time and ducks music. **Revisit this for online matches:** one browser losing focus must not freeze its opponent's match or create divergent clocks.
- Keep the browser loop nonblocking. No blocking sleeps, nested wait loops, or desktop-only dependencies in shared browser code.
- Keep source private if desired; publish only build output. Browser-delivered WASM, JavaScript, fonts, and assets remain downloadable by visitors.

## Milestone 1: room-code multiplayer

### Player flow

1. Choose local/NPC play or online play from the lobby.
2. Create a room, enter a display name, and receive a shareable code.
3. A second player enters the code and their display name.
4. Both players see who has joined and which mode is selected.
5. Once both are ready, start the same match on both devices.
6. Show the same scores, role changes, timer, and final outcome to both players.
7. Offer a rematch or return to the lobby.

### Implementation work

- Select and validate a browser-compatible real-time backend and transport before committing to an architecture.
- Cloudflare Workers, per-room Durable Objects, WebSockets, and D1 are **candidates to investigate**, not a finalized design. Check current platform capabilities, costs, deployment requirements, and Rust/WASM integration at implementation time.
- Establish a single authority for room state, simulation, random spawns, scores, and match time. Do not let two clients independently decide results.
- Define versioned messages for joining, readiness, inputs, state snapshots, rematches, leaving, errors, and match completion.
- Plan latency handling, input sequencing, reconnects, prediction/reconciliation if needed, and behavior under missing or delayed messages.
- Handle invalid/expired codes, full rooms, opponent departure, and connection loss with clear UI.
- Define who chooses the mode and when lobby settings become locked.
- Keep the current deterministic fixed-step movement and existing fair-spawn rules; do not introduce network waits into rendering.

### Completion checks

- Two separate browsers/devices can create/join a room using the code.
- Players can use their own names and complete both game modes.
- Scores, coin/flag locations, CTF halves, and results agree on both devices.
- Invalid codes and disconnections are handled without hanging the game.
- Local and NPC matches still work as before.

## Milestone 2: persistent game log and leaderboard

### Desired behavior

- Record games and their outcomes in durable storage rather than browser local storage.
- After a battle ends, its result appears automatically in the site's match log and updates the applicable leaderboard.
- Show both players' names, mode, score, winner/draw, and when the match took place.
- Provide a readable match-history area and leaderboard that fit the existing simple game-first design.
- Scope the request to **all games being logged**: decide explicitly how online, same-device, and NPC games are recorded and labeled, and which categories contribute to rankings.

### Data and reliability work

- Define a match record with a unique match ID, player identifiers and display-name snapshots, mode, play category, start/end timestamps, scores, outcome, completion status, and game/rules version.
- Decide how to record abandoned games, forfeits, and interrupted sessions; do not silently count them as normal completed wins.
- Save a final result exactly once, including when clients reconnect, messages repeat, or completion requests are retried.
- Online ranked results should come from the authoritative match service; visitors must not be able to submit arbitrary winning scores.
- Keep credentials and database access on the server; expose suitable public read endpoints for history and standings.
- Plan pagination, ranking tie-breaks, and refreshing results after a completed match. Avoid constant unnecessary polling.
- Track separate modes/play categories as needed so NPC practice or local play does not accidentally distort online rankings.

### Decisions to make with Nicolas

- **Identity:** guest display names only, persistent guest profiles, or signed-in accounts? A name alone does not uniquely identify a person.
- **Ranking:** wins/losses/draws, points, win rate, rating, or another system? One leaderboard per mode or combined?
- **Logging:** how should local/NPC games, abandoned matches, and forfeits appear and affect rankings?
- **Rematches:** same room/opponent/settings, and should starting roles alternate?
- **Disconnects/focus:** reconnect grace period, forfeits, and whether any mutually agreed online pause is allowed.
- **Visibility:** public battle history or additional private-room/history options?

## Milestone 3: sprites, sound, and quality of life

### Art and sprite sheets

- Review new assets with Nicolas before choosing replacements or additional characters.
- Support different sprites without assuming every sheet has the same frame size, count, or animation rows.
- Consider explicit sprite-sheet metadata and character definitions rather than scattered frame constants.
- Keep collision sizes and gameplay balance independent of cosmetic artwork.
- Preserve idle/walk/run/jump/fall/dash animations, facing direction, readable names, and spawn effects.
- The current coin is `assets/coin.png`: **five 16×16 frames in an 80×16 sheet**, rendered at 30×30 pixels. An earlier incorrect eight-frame count caused blank animation frames; validate sheet layouts when changing assets.
- Validate transparency, pixel filtering, browser loading, and licensing/credits for new assets.

### Audio

- Add more SFX for events such as coin pickups, flag captures, match starts/ends, and role reversals as assets become available.
- Avoid replaying sounds when authoritative network events are duplicated or state is reconciled.
- Preserve muted-by-default music, volume controls, and browser user-gesture audio activation.
- Consider separate SFX volume/mute controls if Nicolas wants them.

### Quality of life

- Collect Nicolas's specific QoL requests before implementing speculative features.
- Prioritize legibility, room-code copying, clear connection status, rematches, name entry, and useful error messages.
- Check input/focus behavior, browser resizing/fullscreen, and performance under real network conditions.
- Retain existing game-mode selection, match summaries, fair spawns, and keyboard controls.

## Useful commands

```powershell
# Desktop
cargo run

# Rust verification
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown -- -D warnings

# Rebuild and inspect the web package
node scripts/build-web.mjs
node scripts/check-web.mjs
node --test scripts/web.test.mjs

# Run the site locally, then open http://127.0.0.1:8080
node scripts/serve-web.mjs

# With the server running (Node 22+, Chromium/Edge)
node scripts/browser-smoke.mjs
node scripts/browser-smoke.mjs --benchmark
```

The browser check uses its own headless profile and covers startup, greyscale activation/resume, fonts, muted-by-default music and mute toggling, themes, read-only notes, both modes, and resizing. It is not yet a multiplayer test. Browser screenshots and performance samples are in ignored `target/`.

## Suggested next-session order

1. Confirm identity/ranking expectations and the first online room flow.
2. Inspect hosting/account state and choose the backend after validating current options.
3. Prove create/join and real-time communication between two browsers.
4. Integrate one authoritative complete match, then both modes and disconnect/rematch behavior.
5. Persist results and display the battle log/leaderboard automatically.
6. Take the sprite/SFX/QoL requests in small, reviewable updates.

Do not represent these plans as completed work or publish/deploy anything merely because it is listed here.
