const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const { getCurrentWebview } = window.__TAURI__.webview;

const VIDEO_EXTS = ['mp4', 'webm', 'mov', 'mkv', 'avi'];
const AUDIO_EXTS = ['mp3', 'wav', 'ogg', 'flac', 'm4a'];
const ALL_EXTS = [...VIDEO_EXTS, ...AUDIO_EXTS];

function getExtension(pathOrName) {
  const clean = pathOrName.split(/[\\/]/).pop();
  const dot = clean.lastIndexOf('.');
  return dot > -1 ? clean.slice(dot + 1).toLowerCase() : '';
}
function isMediaFile(pathOrName) {
  return ALL_EXTS.includes(getExtension(pathOrName));
}
function baseName(path) {
  return path.split(/[\\/]/).pop();
}
function nameWithoutExt(fileName) {
  const dot = fileName.lastIndexOf('.');
  return dot > 0 ? fileName.slice(0, dot) : fileName;
}

const translations = {
  en: {
    tab_home: 'Home',
    tab_settings: 'Settings',
    tab_support: 'Support',
    home_screamers_title: 'Screamers',
    home_screamers_desc: 'Random jump scares during your stream',
    home_interval_label: 'Scare interval (minutes)',
    home_interval_hint: 'We recommend a 3-minute minimum so you have time to get into the game before it finds you.',
    home_chatvote_title: 'Chat Vote',
    home_chatvote_desc: 'Let viewers vote for the next scare',
    home_obs_label: 'Connect the widget to OBS',
    home_obs_step1: 'Open OBS → Sources → click +',
    home_obs_step2: 'Select "Browser"',
    home_obs_step3: 'Paste the widget link (generated here later)',
    home_obs_step4: 'Set size to 400×200 and place it in a corner',
    home_obs_warning: "⚠️ Please don't peek at OBS — let's keep the surprise, the fear and the fun intact!",
    home_list_title: 'Screamers',
    settings_language: 'Language',
    settings_connections: 'Connections',
    settings_twitch: 'Twitch',
    settings_tiktok: 'TikTok',
    settings_youtube: 'YouTube',
    settings_soon: 'Coming soon',
    settings_not_connected: 'Not connected',
    settings_connected: 'Connected',
    settings_connect: 'Connect',
    settings_disconnect: 'Disconnect',
    support_title: 'Support the project',
    support_hint: "SoS is free. If you'd like to help keep it that way, you can send a tip below — no pressure, ever.",
    lang_modal_title: 'Choose your language',
    lang_modal_subtitle: 'You can change this anytime in Settings',
    add_dropzone_hint: 'To add your own files, drag them here, or choose manually.',
    add_dropzone_hint_error: 'Only video and audio files are supported.',
    name_modal_title: 'Name this file',
    name_modal_placeholder: 'Enter a name for the file',
    name_modal_confirm: 'Add',
    name_modal_progress_multi: (i, n) => `File ${i} of ${n}`,
  },
  ru: {
    tab_home: 'Главная',
    tab_settings: 'Настройки',
    tab_support: 'Поддержать',
    home_screamers_title: 'Скримеры',
    home_screamers_desc: 'Случайные скримеры во время стрима',
    home_interval_label: 'Интервал скримера (в минутах)',
    home_interval_hint: 'Рекомендуем минимум 3 минуты, чтобы успеть погрузиться в игру, прежде чем он тебя найдёт.',
    home_chatvote_title: 'Голосование чата',
    home_chatvote_desc: 'Пусть зрители голосуют за следующий скример',
    home_obs_label: 'Подключение виджета к OBS',
    home_obs_step1: 'Открой OBS → Источники → нажми +',
    home_obs_step2: 'Выбери «Браузер»',
    home_obs_step3: 'Вставь ссылку на виджет (сгенерируется здесь позже)',
    home_obs_step4: 'Задай размер 400×200 и размести в углу экрана',
    home_obs_warning: '⚠️ Пожалуйста, не подглядывай в OBS — давай сохраним интригу, страх и веселье!',
    home_list_title: 'Скримеры',
    settings_language: 'Язык',
    settings_connections: 'Подключения',
    settings_twitch: 'Twitch',
    settings_tiktok: 'TikTok',
    settings_youtube: 'YouTube',
    settings_soon: 'Скоро',
    settings_not_connected: 'Не подключено',
    settings_connected: 'Подключено',
    settings_connect: 'Подключить',
    settings_disconnect: 'Отключить',
    support_title: 'Поддержать проект',
    support_hint: 'SoS бесплатен. Если хочешь помочь ему остаться таким — можешь оставить донат ниже. Это совсем не обязательно.',
    lang_modal_title: 'Выберите язык',
    lang_modal_subtitle: 'Вы всегда сможете изменить это в настройках',
    add_dropzone_hint: 'Для добавления своих файлов перетащите их сюда, либо выберите вручную.',
    add_dropzone_hint_error: 'Поддерживаются только видео и аудио файлы.',
    name_modal_title: 'Назовите файл, оно будет использоваться в голосовани',
    name_modal_placeholder: 'Введите название для файла',
    name_modal_confirm: 'Добавить',
    name_modal_progress_multi: (i, n) => `Файл ${i} из ${n}`,
  },
};

let currentLang = 'en';

function applyLanguage(lang) {
  currentLang = lang;
  const dict = translations[lang];

  document.querySelectorAll('[data-i18n]').forEach((el) => {
    const key = el.dataset.i18n;
    if (dict[key]) el.textContent = dict[key];
  });

  document.querySelectorAll('[data-i18n-placeholder]').forEach((el) => {
    const key = el.dataset.i18nPlaceholder;
    if (dict[key]) el.placeholder = dict[key];
  });

  document.documentElement.lang = lang;
  document.getElementById('langSelect').value = lang;

  refreshConnectionTexts();
  localStorage.setItem('sos_lang', lang);
}

const tabs = document.querySelectorAll('.tab-btn');
const indicator = document.getElementById('tabsIndicator');
const pages = document.querySelectorAll('.page');

function moveIndicatorTo(btn) {
  const nav = btn.parentElement;
  const navRect = nav.getBoundingClientRect();
  const btnRect = btn.getBoundingClientRect();
  indicator.style.width = `${btnRect.width}px`;
  indicator.style.transform = `translateX(${btnRect.left - navRect.left}px)`;
}

function switchTab(tabName) {
  tabs.forEach((t) => t.classList.toggle('active', t.dataset.tab === tabName));
  pages.forEach((p) => p.classList.toggle('active', p.id === `page-${tabName}`));
}

tabs.forEach((btn) => {
  btn.addEventListener('click', () => {
    switchTab(btn.dataset.tab);
    moveIndicatorTo(btn);
  });
});

function initIndicator() {
  const activeBtn = document.querySelector('.tab-btn.active');
  if (activeBtn) moveIndicatorTo(activeBtn);
}
window.addEventListener('DOMContentLoaded', initIndicator);
window.addEventListener('load', initIndicator);

function setupTogglePanel(checkboxId, panelId) {
  const checkbox = document.getElementById(checkboxId);
  const panel = document.getElementById(panelId);
  checkbox.addEventListener('change', () => {
    panel.classList.toggle('open', checkbox.checked);
  });
}
setupTogglePanel('toggleScreamers', 'panelScreamers');
setupTogglePanel('toggleChatVote', 'panelChatVote');

const screamerListEl = document.getElementById('screamerList');
const addScreamerBtn = document.getElementById('addScreamerBtn');
const refreshScreamersBtn = document.getElementById('refreshScreamersBtn');

let screamers = [];

async function loadScreamers() {
  try {
    screamers = await invoke('list_screamers');
  } catch (e) {
    console.error('Failed to load screamers:', e);
    screamers = [];
  }
  renderScreamers();
}

function renderScreamers() {
  screamerListEl.innerHTML = '';
  screamers.forEach((s) => screamerListEl.appendChild(createScreamerItem(s)));
}

refreshScreamersBtn.addEventListener('click', async () => {
  if (refreshScreamersBtn.classList.contains('spinning')) return;
  refreshScreamersBtn.classList.add('spinning');
  refreshScreamersBtn.disabled = true;

  const startedAt = Date.now();
  await loadScreamers();

  const minSpinMs = 500;
  const elapsed = Date.now() - startedAt;
  if (elapsed < minSpinMs) {
    await new Promise((resolve) => setTimeout(resolve, minSpinMs - elapsed));
  }

  refreshScreamersBtn.classList.remove('spinning');
  refreshScreamersBtn.disabled = false;
});

function createScreamerItem(screamer) {
  const item = document.createElement('div');
  item.className = 'screamer-item';
  item.dataset.id = screamer.id;

  item.innerHTML = `
    <span class="screamer-name">${screamer.name}</span>
    <input class="screamer-name-input hidden" type="text" value="${screamer.name}">
    <div class="screamer-actions">
      <button class="icon-btn-small test-btn" title="Test" type="button">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" stroke="none"><path d="M8 5v14l11-7Z"/></svg>
      </button>
      <button class="icon-btn-small rename-btn" title="Rename" type="button">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>
      </button>
      <button class="icon-btn-small delete-btn" title="Delete" type="button">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/></svg>
      </button>
    </div>
  `;

  const nameSpan = item.querySelector('.screamer-name');
  const nameInput = item.querySelector('.screamer-name-input');
  const testBtn = item.querySelector('.test-btn');
  const renameBtn = item.querySelector('.rename-btn');
  const deleteBtn = item.querySelector('.delete-btn');

  testBtn.addEventListener('click', async () => {
    try {
      await invoke('trigger_scare', { id: screamer.id });
    } catch (e) {
      console.error('Test scare failed:', e);
    }
  });

  renameBtn.addEventListener('click', () => {
    item.classList.add('editing');
    nameInput.classList.remove('hidden');
    nameInput.focus();
    nameInput.select();
  });

  async function confirmRename() {
    const newName = nameInput.value.trim();
    nameInput.classList.add('hidden');
    item.classList.remove('editing');

    if (!newName || newName === screamer.name) {
      nameInput.value = screamer.name;
      return;
    }

    try {
      const updated = await invoke('rename_screamer', { id: screamer.id, newName });
      screamer.id = updated.id;
      screamer.name = updated.name;
      nameSpan.textContent = screamer.name;
      item.dataset.id = screamer.id;
      nameInput.value = screamer.name;
    } catch (e) {
      console.error('Rename failed:', e);
      nameInput.value = screamer.name;
    }
  }

  nameInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') confirmRename();
    if (e.key === 'Escape') {
      nameInput.value = screamer.name;
      nameInput.classList.add('hidden');
      item.classList.remove('editing');
    }
  });
  nameInput.addEventListener('blur', confirmRename);

  deleteBtn.addEventListener('click', async () => {
    try {
      await invoke('delete_screamer', { id: screamer.id });
      screamers = screamers.filter((s) => s.id !== screamer.id);
      renderScreamers();
    } catch (e) {
      console.error('Delete failed:', e);
    }
  });

  return item;
}

const addModal = document.getElementById('addModal');
const addModalClose = document.getElementById('addModalClose');
const dropzone = document.getElementById('dropzone');
const dropzoneHint = document.getElementById('dropzoneHint');

function openAddModal() {
  addModal.classList.add('visible');
}
function closeAddModal() {
  addModal.classList.remove('visible');
  dropzone.classList.remove('drag-over');
}

addScreamerBtn.addEventListener('click', openAddModal);
addModalClose.addEventListener('click', closeAddModal);
addModal.addEventListener('click', (e) => {
  if (e.target === addModal) closeAddModal();
});

let dropzoneHintTimer = null;
function flashDropzoneError() {
  const dict = translations[currentLang];
  clearTimeout(dropzoneHintTimer);
  dropzoneHint.textContent = dict.add_dropzone_hint_error;
  dropzoneHint.style.color = '#ff5577';
  dropzoneHintTimer = setTimeout(() => {
    dropzoneHint.textContent = dict.add_dropzone_hint;
    dropzoneHint.style.color = '';
  }, 2200);
}

dropzone.addEventListener('click', async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Media', extensions: ALL_EXTS }],
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    startNamingQueue(paths);
  } catch (e) {
    console.error('File dialog failed:', e);
  }
});

getCurrentWebview().onDragDropEvent((event) => {
  if (!addModal.classList.contains('visible')) return;

  if (event.payload.type === 'over') {
    dropzone.classList.add('drag-over');
  } else if (event.payload.type === 'drop') {
    dropzone.classList.remove('drag-over');
    startNamingQueue(event.payload.paths || []);
  } else {
    dropzone.classList.remove('drag-over');
  }
});

const nameModal = document.getElementById('nameModal');
const nameModalClose = document.getElementById('nameModalClose');
const nameModalProgress = document.getElementById('nameModalProgress');
const nameModalOriginal = document.getElementById('nameModalOriginal');
const nameModalInput = document.getElementById('nameModalInput');
const nameModalConfirm = document.getElementById('nameModalConfirm');

let namingQueue = [];
let namingIndex = 0;
let resolvedFiles = [];

function startNamingQueue(paths) {
  const validPaths = paths.filter(isMediaFile);
  if (validPaths.length === 0) {
    flashDropzoneError();
    return;
  }
  namingQueue = validPaths;
  namingIndex = 0;
  resolvedFiles = [];
  closeAddModal();
  showNamingStep();
}

function showNamingStep() {
  const path = namingQueue[namingIndex];
  const original = baseName(path);
  nameModalOriginal.textContent = original;
  nameModalInput.value = nameWithoutExt(original);
  nameModalProgress.textContent = namingQueue.length > 1
    ? translations[currentLang].name_modal_progress_multi(namingIndex + 1, namingQueue.length)
    : '';
  nameModal.classList.add('visible');
  nameModalInput.focus();
  nameModalInput.select();
}

async function confirmNamingStep() {
  const path = namingQueue[namingIndex];
  const original = baseName(path);
  const ext = getExtension(original);
  const customBase = nameModalInput.value.trim();
  const finalName = customBase ? `${customBase}.${ext}` : original;
  resolvedFiles.push({ path, name: finalName });

  namingIndex += 1;
  if (namingIndex < namingQueue.length) {
    showNamingStep();
  } else {
    await finishNamingQueue();
  }
}

function cancelNamingQueue() {
  nameModal.classList.remove('visible');
  namingQueue = [];
  namingIndex = 0;
  resolvedFiles = [];
}

async function finishNamingQueue() {
  nameModal.classList.remove('visible');
  const filesToAdd = resolvedFiles;
  resolvedFiles = [];
  namingQueue = [];
  namingIndex = 0;

  try {
    screamers = await invoke('add_screamer_files', { files: filesToAdd });
    renderScreamers();
  } catch (e) {
    console.error('Failed to add files:', e);
  }
}

nameModalConfirm.addEventListener('click', confirmNamingStep);
nameModalInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') confirmNamingStep();
  if (e.key === 'Escape') cancelNamingQueue();
});
nameModalClose.addEventListener('click', cancelNamingQueue);
nameModal.addEventListener('click', (e) => {
  if (e.target === nameModal) cancelNamingQueue();
});

const langSelect = document.getElementById('langSelect');
langSelect.addEventListener('change', () => {
  applyLanguage(langSelect.value);
});

const connectionState = { twitch: false, tiktok: false };

function refreshConnectionTexts() {
  const dict = translations[currentLang];
  const twitchBtn = document.getElementById('twitchBtn');
  const twitchStatus = document.getElementById('twitchStatus');
  const tiktokBtn = document.getElementById('tiktokBtn');
  const tiktokStatus = document.getElementById('tiktokStatus');

  twitchBtn.textContent = connectionState.twitch ? dict.settings_disconnect : dict.settings_connect;
  twitchBtn.classList.toggle('connected', connectionState.twitch);
  twitchStatus.textContent = connectionState.twitch ? dict.settings_connected : dict.settings_not_connected;
  twitchStatus.classList.toggle('connected', connectionState.twitch);

  tiktokBtn.textContent = connectionState.tiktok ? dict.settings_disconnect : dict.settings_connect;
  tiktokBtn.classList.toggle('connected', connectionState.tiktok);
  tiktokStatus.textContent = connectionState.tiktok ? dict.settings_connected : dict.settings_not_connected;
  tiktokStatus.classList.toggle('connected', connectionState.tiktok);
}

function setupConnectButton(platform, btnId) {
  const btn = document.getElementById(btnId);
  btn.addEventListener('click', () => {
    connectionState[platform] = !connectionState[platform];
    refreshConnectionTexts();
  });
}
setupConnectButton('twitch', 'twitchBtn');
setupConnectButton('tiktok', 'tiktokBtn');

const langModal = document.getElementById('langModal');

function initLanguage() {
  const saved = localStorage.getItem('sos_lang');
  if (saved && translations[saved]) {
    applyLanguage(saved);
  } else {
    applyLanguage('en');
    langModal.classList.add('visible');
  }
}

document.querySelectorAll('.lang-option').forEach((btn) => {
  btn.addEventListener('click', () => {
    applyLanguage(btn.dataset.lang);
    langModal.classList.remove('visible');
  });
});

initLanguage();
loadScreamers();