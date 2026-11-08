// Default to dark on every device, even when browser storage is unavailable.
// An explicit light-mode choice still takes precedence on later visits.
document.documentElement.dataset.theme = 'dark';
try {
  if (localStorage.getItem('pyctf.theme') === 'light') document.documentElement.dataset.theme = 'light';
} catch { /* Storage may be unavailable in a private or embedded browser. */ }
