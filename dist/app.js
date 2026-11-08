/* The vendored Macroquad bundle owns WebGL and the normal requestAnimationFrame loop. */
(() => {
  const themeToggle = document.querySelector('#theme-toggle');
  function updateThemeButton() {
    const dark = document.documentElement.dataset.theme === 'dark';
    themeToggle.setAttribute('aria-pressed', String(dark));
    themeToggle.setAttribute('aria-label', dark ? 'Switch to light mode' : 'Switch to dark mode');
    themeToggle.title = themeToggle.getAttribute('aria-label');
  }
  updateThemeButton();
  themeToggle.addEventListener('click', () => {
    const theme = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
    document.documentElement.dataset.theme = theme;
    try { localStorage.setItem('pyctf.theme', theme); } catch { /* Theme still works without storage. */ }
    updateThemeButton();
  });

  const notes = document.querySelector('#patch-notes');
  fetch('updates.txt').then(response => {
    if (!response.ok) throw new Error('No published notes');
    return response.text();
  }).then(text => { notes.textContent = text; })
    .catch(() => { notes.textContent = 'No updates to show yet.'; });

  const canvas = document.querySelector('#glcanvas');
  const stage = document.querySelector('#stage');
  const overlay = document.querySelector('#overlay');
  const status = document.querySelector('#status');
  const hint = document.querySelector('#hint');
  let ready = false;
  let started = false;
  let failed = false;

  function pause() {
    if (!ready || !started || failed) return;
    wasm_exports.pyctf_set_active(0);
    // Clear held movement keys when leaving the canvas or browser tab.
    for (const key of [65, 68, 87, 83, 81, 69, 74, 76, 73, 75, 85, 79]) {
      wasm_exports.key_up(key, 0);
    }
    stage.classList.add('inactive');
    status.textContent = 'Click in the window to resume';
    overlay.setAttribute('aria-label', status.textContent);
    hint.hidden = true;
    overlay.hidden = false;
  }

  overlay.addEventListener('click', () => {
    if (!ready || failed) return;
    started = true;
    overlay.hidden = true;
    stage.classList.remove('inactive');
    wasm_exports.pyctf_set_active(1);
    canvas.focus();
  });
  window.addEventListener('blur', pause);
  canvas.addEventListener('blur', pause);
  document.addEventListener('visibilitychange', () => { if (document.hidden) pause(); });
  canvas.addEventListener('contextmenu', (event) => event.preventDefault());
  document.querySelector('#fullscreen').addEventListener('click', async () => {
    try {
      if (document.fullscreenElement) await document.exitFullscreen();
      else await document.querySelector('#stage').requestFullscreen();
    } catch {
      document.querySelector('#page-message').textContent = 'Fullscreen is unavailable in this browser. You can still play in the page.';
    }
  });

  function fail(message) {
    failed = true;
    status.textContent = 'Couldn’t start the game';
    hint.textContent = message;
    hint.hidden = false;
    overlay.disabled = true;
    stage.classList.add('inactive');
    overlay.hidden = false;
  }
  window.addEventListener('error', (event) => fail(event.message || 'Reload the page and try again.'));
  window.addEventListener('unhandledrejection', () => fail('Reload the page and check that your browser supports WebAssembly and WebGL.'));

  if (location.protocol === 'file:') {
    fail('Serve this folder over HTTP. From the project run: npm run web:serve');
    return;
  }
  const startedAt = performance.now();
  function waitForGame() {
    if (failed) return;
    if (typeof wasm_exports !== 'undefined' && wasm_exports?.pyctf_ready?.()) {
      ready = true;
      status.textContent = 'Click in the game window to play';
      overlay.setAttribute('aria-label', status.textContent);
      overlay.disabled = false;
      return;
    }
    if (performance.now() - startedAt > 60000) {
      fail('Loading took too long. Reload the page, or check the browser console for download or WebGL errors.');
      return;
    }
    requestAnimationFrame(waitForGame);
  }
  load('pyctf.wasm');
  requestAnimationFrame(waitForGame);
})();
