# Macroquad browser runtime

`mq_js_bundle.js` is vendored unchanged from Macroquad **0.4.16** (`js/mq_js_bundle.js` in the crates.io distribution).
Source commit: `5e9b5ca912ac65962c05c0da842a4a70eaae34b9`.

This bundle includes Miniquad's WebGL/browser loader, the quad-snd audio plugin, sapp-jsutils and quad-net shims. Do not load `audio.js` again. No third-party CDN is needed at runtime.

`../runtime-compat.js` declares the global used by the bundle's optional quad-net adapter to avoid its strict-mode ReferenceError. Keep it before the bundle in the HTML.

Upstream: https://github.com/not-fl3/macroquad/tree/5e9b5ca912ac65962c05c0da842a4a70eaae34b9/js

Macroquad is dual licensed under MIT or Apache-2.0; both license texts are retained here. When updating Macroquad, update the bundle from the matching crate, retain its license notices, and repeat browser checks.

`ProggyClean.ttf` is the unchanged default font from Macroquad 0.4.16 (`src/ProggyClean.ttf`). The page uses this same font as the game. It is by Tristan Grimmer; its MIT license is retained as `LICENSE-ProggyClean.txt`. Upstream: https://github.com/bluescan/proggyfonts
