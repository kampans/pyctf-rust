# Browser build and hosting

The existing Rust game now builds as a static WebAssembly website. Desktop `cargo run` remains supported. This preparation is local; nothing has been published or connected to a hosting account.

## Build and play locally

Install Rust with Cargo and Node.js 20 or newer. From the repository root:

```powershell
rustup target add wasm32-unknown-unknown
node scripts/build-web.mjs
node scripts/serve-web.mjs
```

Open **http://127.0.0.1:8080**. After loading, click anywhere in the greyscale game window to enable audio and keyboard input. Choose a mode and opponent in the game lobby. Use Ctrl+C in the server terminal to stop it. Choose another port with `node scripts/serve-web.mjs --port 8081`.

No npm dependencies or `npm install` are needed. Optional aliases are `npm run web:build`, `npm run web:serve`, `npm run web:check`, and `npm run test:web`; on Windows with restricted PowerShell script execution, use `npm.cmd` or the direct `node` commands above.

Do not open `index.html` using `file://`; WASM and texture loading need an HTTP server. Rebuild after changing Rust, assets, or files in `web/`. `dist/` is generated and ignored by Git.

## What is prepared

- Both modes share the same gameplay code as desktop, including the NPC, match summaries, and menu-only music controls.
- Rendering and pointer input use a 1600×900 logical arena that scales with the canvas and keeps its aspect ratio, including fullscreen.
- Movement uses a fixed 60 Hz simulation, independent of browser refresh rate, with player rendering interpolated between ticks for smoother motion on faster displays. Slow frames have bounded catch-up.
- NPC routes are precomputed once for the fixed arena; the NPC does not allocate queues or repeat graph searches during play. Coin spawn selection reuses candidate storage and samples valid pairs without storing the entire pair list. HUD text/layout is refreshed only when its values change, and the opaque arena does not get an extra full-screen fill behind it.
- Switching tabs, changing window focus, or moving focus outside the canvas makes the game greyscale with “Click in the window to resume.” Match time and movement remain frozen in the background, and music is ducked. Clicking anywhere in the game restores color and play, without simulating time spent away or leaving movement keys held. No sleep or blocking game loop is used.
- Sound files are embedded in the WASM. Music starts muted on both desktop and web; unmute in the lobby to listen at the chosen volume (35% by default). Browser playback initializes after clicking the game window and uses the Web Audio plugin included in the vendored Macroquad runtime. Native playback initializes immediately. Jump/dash effects remain enabled. The page and browser game both use Proggy Clean, bundled locally with its license, with colors drawn from the lobby and players.
- Random spawns use Macroquad's cross-platform generator, seeded at startup; there is no `getrandom` / wasm-bindgen dependency requiring a different loader.
- The JavaScript runtime is vendored from Macroquad 0.4.16, including audio, so the finished page makes no CDN requests. Keep it matched to the Rust dependency when updating; see `web/vendor/README.md` and its licenses.
- `.cargo/config.toml` applies a WASM-only linker setting for quad-snd's JavaScript-provided functions. The browser check verifies runtime integration; native linking is unaffected.

## Validate

```powershell
cargo test --locked
cargo clippy --all-targets -- -D warnings
node scripts/build-web.mjs
node scripts/check-web.mjs
node --test scripts/web.test.mjs
```

The package check validates relative HTML references, WASM exports/imports and textures. The HTTP test checks WASM MIME type, HEAD requests, missing-file handling, and path containment. Rust tests cover match timing, CTF's single role reversal, fair coin spawns, NPC navigation, viewport proportions and fixed-rate simulation.

With the local server running, an optional real Chromium check is:

```powershell
node scripts/browser-smoke.mjs
```

It defaults to Edge's standard Windows install path, or `chromium` on other systems. Set `CHROME_PATH` to another Chromium executable, and `WEB_URL` to test a different server URL. The check launches a separate headless browser with audio output muted, verifies a running AudioContext and source playback, clicks through scaled menus, and tests focus pause/resume and resizing. Screenshots and the isolated browser profile go under ignored `target/`. This confirms audio pipeline activity, not your selected speakers or hardware volume.

The optional browser check needs **Node.js 22 or newer** for its built-in WebSocket client. Add `--benchmark` to sample frame-callback CPU duration and frame intervals during three seconds of Coin Race with the NPC:

```powershell
node scripts/browser-smoke.mjs --benchmark
```

Results are written to `target/web-perf.json`. This uses a software GPU in an isolated headless browser: use it for repeatable checks, not as a prediction of FPS on every device. Release builds enable thin LTO and a single code generation unit; build time can be longer, and browser downloads use the optimized output.

The GitHub Actions **Web build** workflow runs the build and automated package checks, then uploads a `pyctf-web` artifact containing `dist/`. It does not deploy anything. The optional browser smoke check is run separately.

## Page appearance and patch notes

The game appears first, capped at 1360 pixels wide and 84% of the viewport height on desktop. The title and read-only updates area sit below it, with red and blue accents matching the players. Dark mode is the default on all devices, including when browser storage is unavailable; the bottom-right moon/sun button switches themes and remembers an explicit choice. The CSS is grouped into commented sections with one declaration per line.

The updates area displays `web/updates.txt` as plain text. To publish changes, edit that file, rebuild with `node scripts/build-web.mjs`, then upload the updated `dist/` contents. Visitors cannot edit the notes, and old local drafts are ignored.

## Host later

Upload the **contents of `dist/`** to a static host. The output is:

```text
dist/
  index.html
  styles.css
  app.js
  theme.js
  updates.txt
  runtime-compat.js
  pyctf.wasm
  assets/        # textures
  vendor/        # matching browser runtime and license notices
```

Serve `.wasm` as **`application/wasm`**, JavaScript as JavaScript, and PNGs as `image/png`. Keep missing asset requests as 404s instead of rewriting them to the homepage. All URLs are relative, so the same package can live at a domain root or a project subdirectory. Use HTTPS in production. When embedding in an iframe, allow fullscreen and autoplay, and keep the click-to-play gesture.

The release WASM is approximately 1.9 MiB before HTTP compression. Serving gzip or Brotli will reduce transfer size. No backend, database, WebAssembly threads, COOP/COEP headers, or account service is required for the current game.

The supported input is **desktop keyboard and pointer**, with two players sharing one keyboard or P2 controlled by the NPC. The layout scales on smaller displays, but touchscreen game controls and online multiplayer have not been added. Chromium startup/rendering/audio have been checked; Firefox and Safari, mobile audio behavior, and a production host should be checked when choosing the actual deployment target.

Build approach: [Macroquad's official WASM instructions](https://github.com/not-fl3/macroquad#wasm) and [browser integration documentation](https://macroquad.rs/articles/wasm/).
