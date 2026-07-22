const { invoke, convertFileSrc } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

const stage = document.getElementById('stage');
let closed = false;

const WATCHDOG_MS = 20000;
const watchdogTimer = setTimeout(() => {
  console.warn('Scare watchdog: force-closing overlay after timeout');
  closeOverlay();
}, WATCHDOG_MS);

async function closeOverlay() {
  if (closed) return;
  closed = true;
  clearTimeout(watchdogTimer);
  try {
    await getCurrentWindow().close();
  } catch (e) {
    console.error('Failed to close overlay window:', e);
  }
}

async function start() {
  let media = null;
  try {
    media = await invoke('take_scare_media');
  } catch (e) {
    console.error('Failed to fetch scare media:', e);
  }

  if (!media) {
    closeOverlay();
    return;
  }

  const el = document.createElement(media.kind === 'video' ? 'video' : 'audio');
  el.src = convertFileSrc(media.path);
  el.autoplay = true;
  el.controls = false;
  el.style.display = media.kind === 'video' ? 'block' : 'none';

  el.addEventListener('ended', closeOverlay);
  el.addEventListener('error', (e) => {
    console.error('Playback error:', e);
    closeOverlay();
  });

  stage.appendChild(el);
  el.play().catch((e) => {
    console.error('Autoplay was blocked:', e);
    closeOverlay();
  });
}

window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closeOverlay();
});

start();