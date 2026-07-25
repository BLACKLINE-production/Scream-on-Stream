const { invoke, convertFileSrc } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const stage = document.getElementById('stage');

const WATCHDOG_MS = 20000;
let watchdogTimer = null;
let currentEl = null;

function clearStage() {
  if (watchdogTimer) {
    clearTimeout(watchdogTimer);
    watchdogTimer = null;
  }
  if (currentEl) {
    try { currentEl.pause(); } catch (e) {}
    currentEl.remove();
    currentEl = null;
  }
  stage.innerHTML = '';
}

function armWatchdog() {
  if (watchdogTimer) clearTimeout(watchdogTimer);
  watchdogTimer = setTimeout(() => {
    console.warn('Scare watchdog: clearing overlay after timeout');
    clearStage();
  }, WATCHDOG_MS);
}

function playMedia(media) {
  if (!media) return;
  clearStage();

  const el = document.createElement(media.kind === 'video' ? 'video' : 'audio');
  el.src = convertFileSrc(media.path);
  el.autoplay = true;
  el.controls = false;
  el.volume = Math.min(1, Math.max(0, typeof media.volume === 'number' ? media.volume : 1));
  el.style.display = media.kind === 'video' ? 'block' : 'none';

  el.addEventListener('ended', clearStage);
  el.addEventListener('error', (e) => {
    console.error('Playback error:', e);
    clearStage();
  });

  currentEl = el;
  stage.appendChild(el);
  armWatchdog();

  el.play().catch((e) => {
    console.error('Autoplay was blocked:', e);
    clearStage();
  });
}

async function start() {
  try {
    const pending = await invoke('take_scare_media');
    if (pending) playMedia(pending);
  } catch (e) {
    console.error('Failed to fetch pending scare media:', e);
  }

  await listen('scare://play', (event) => playMedia(event.payload));
  await listen('scare://stop', () => clearStage());
}

window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') clearStage();
});

start();