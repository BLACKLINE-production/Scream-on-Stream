const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const { getCurrentWebview } = window.__TAURI__.webview;
const { openUrl } = window.__TAURI__.opener;

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
    home_screamers_warning: '⚠️ Use "Borderless Window" display mode in your game — otherwise the screamers may not work correctly.',
    home_chatvote_title: 'Chat Vote',
    home_chatvote_desc: 'Let viewers vote for the next scare',
    home_widget_compat: "Both guides work with the same widget — connect one or both, they run together with no conflict.",
    home_obs_label: 'Connect the widget to OBS',
    home_obs_step1: 'Connect your account in the "Settings" section',
    home_obs_step2: 'Open OBS → Sources → click +',
    home_obs_step3: 'Select "Browser"',
    home_obs_step4: 'Paste the link to the widget (located below)',
    home_obs_step5: 'Set size to 400×300 and place it in a corner',
    home_tiktok_label: 'Connect the widget to TikTok Live Studio',
    home_tiktok_step1: 'Connect your account in the "Settings" section',
    home_tiktok_step2: 'Open TikTok Live Studio → top-left square → click "Add Source"',
    home_tiktok_step3: 'Select "Link" → Add',
    home_tiktok_step4: 'Paste the link to the widget (located below)',
    home_tiktok_step5: 'Set size to 400×300 and place it in a corner',
    home_widget_link_label: 'Link to add the vote widget',
    home_widget_warning: "⚠️ Please don't peek at the widget while it's live — let's keep the surprise, the fear and the fun intact!",
    home_list_title: 'Screamers',
    home_volume_label: 'Master Volume',
    home_volume_hint: 'Applies to every screamer — videos and sounds alike.',
    settings_language: 'Language',
    settings_autostart: 'Launch on PC startup',
    settings_autostart_hint: "SoS will start automatically every time you turn on your PC — no need to open it manually before you go live.",
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
    home_votetime_label: 'Vote duration (seconds)',
    widget_link_copy: 'Copy link',
    widget_link_copied: 'Copied!',
    twitch_prompt_channel: 'Enter your Twitch channel name (no @):',
    twitch_connect_error: "Couldn't connect to that Twitch channel. Check the name and try again.",
    tiktok_prompt_username: 'Enter your TikTok username (no @):',
    tiktok_prompt_apikey: 'Enter your TikTok API key (free key at tik.tools):',
    tiktok_connect_error: "Couldn't connect. Check your username and API key (get a free one at tik.tools) and make sure you're LIVE.",
    connect_twitch_title: 'Connect Twitch',
    connect_twitch_subtitle: "We'll read your chat to count votes.",
    connect_twitch_label: 'Channel name',
    connect_twitch_placeholder: 'your channel',
    connect_tiktok_title: 'Connect TikTok',
    connect_tiktok_subtitle: "We'll read your chat to count votes.",
    connect_tiktok_label_username: 'TikTok username',
    connect_tiktok_placeholder_username: 'your username',
    connect_tiktok_label_apikey: 'API key',
    connect_tiktok_placeholder_apikey: 'Paste your API key',
    connect_tiktok_hint: 'Get a free key at <a href="https://tik.tools/login" target="_blank" rel="noopener">tik.tools</a> — takes about 30 seconds.',
    connect_modal_confirm: 'Connect',
    connect_modal_connecting: 'Connecting…',
    settings_panic_title: 'Panic Button',
    settings_panic_hint: "Instantly kills whatever scare is currently playing — video, sound, everything — no matter what triggered it. Timers and chat vote keep running underneath.",
    settings_panic_hotkey_label: 'Hotkey',
    settings_panic_change: 'Change',
    settings_panic_cancel_btn: 'Cancel',
    settings_panic_recording: 'Press keys…',
    settings_panic_vote_hint: 'If Chat Vote is on, the next vote round appears about 5 seconds after you use the panic button.',
    settings_panic_error_modifier: 'Pick at least one modifier key (Ctrl, Alt or Shift).',
    settings_panic_error_key: "That key isn't supported — try a letter, number, F-key, or Space.",
    settings_panic_error_taken: 'That combination is already used by another app on this system.',
    update_modal_title: 'New version available?',
    update_modal_text: (version) => `Hey! Thanks for using SoS for about a month now. There's a good chance a new version has come out with cool new features. We recommend checking whether an update is available. Your current version: ${version}`,
    update_modal_check_btn: 'Check for updates',
  },
  ru: {
    tab_home: 'Главная',
    tab_settings: 'Настройки',
    tab_support: 'Поддержать',
    home_screamers_title: 'Скримеры',
    home_screamers_desc: 'Случайные скримеры во время стрима',
    home_interval_label: 'Интервал скримера (в минутах)',
    home_interval_hint: 'Рекомендуем минимум 3 минуты, чтобы успеть погрузиться в игру, прежде чем он тебя найдёт.',
    home_screamers_warning: '⚠️ Используй режим отображения «Оконный без рамки» в игре — иначе скримеры могут срабатывать некорректно.',
    home_chatvote_title: 'Голосование чата',
    home_chatvote_desc: 'Пусть зрители голосуют за следующий скример',
    home_widget_compat: 'Обе инструкции работают с одним и тем же виджетом — можно подключить одну или сразу обе, они не конфликтуют.',
    home_obs_label: 'Подключение виджета к OBS',
    home_obs_step1: 'Подключи аккаунт в разделе "Настройки"',
    home_obs_step2: 'Открой OBS → Источники → нажми +',
    home_obs_step3: 'Выбери «Браузер»',
    home_obs_step4: 'Вставь ссылку на виджет (находится ниже)',
    home_obs_step5: 'Задай размер 400×300 и размести в углу экрана',
    home_tiktok_label: 'Подключение виджета к TikTok Live Studio',
    home_tiktok_step1: 'Подключи аккаунт в разделе "Настройки"',
    home_tiktok_step2: 'Открой TikTok Live Studio → левый верхний квадрат → нажми «Добавить источник»',
    home_tiktok_step3: 'Выбери «Ссылка» → Добавить',
    home_tiktok_step4: 'Вставь ссылку на виджет (находится ниже)',
    home_tiktok_step5: 'Задай размер 400×300 и размести в углу экрана',
    home_widget_link_label: 'Ссылка для добавления голосования',
    home_widget_warning: '⚠️ Пожалуйста, не подглядывай в виджет, пока он в эфире — давай сохраним интригу, страх и веселье!',
    home_list_title: 'Скримеры',
    home_volume_label: 'Общая громкость',
    home_volume_hint: 'Действует на все скримеры — и на видео, и на звуки.',
    settings_language: 'Язык',
    settings_autostart: 'Запуск при включении ПК',
    settings_autostart_hint: 'SoS будет запускаться автоматически при каждом включении компьютера — не нужно открывать его вручную перед стримом.',
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
    name_modal_title: 'Назовите файл, название будет использоваться в голосовани',
    name_modal_placeholder: 'Введите название для файла',
    name_modal_confirm: 'Добавить',
    name_modal_progress_multi: (i, n) => `Файл ${i} из ${n}`,
    home_votetime_label: 'Время голосования (в секундах)',
    widget_link_copy: 'Скопировать ссылку',
    widget_link_copied: 'Скопировано!',
    twitch_prompt_channel: 'Введите название вашего Twitch-канала (без @):',
    twitch_connect_error: 'Не удалось подключиться к этому Twitch-каналу. Проверьте название и попробуйте снова.',
    tiktok_prompt_username: 'Введите ваш TikTok-юзернейм (без @):',
    tiktok_prompt_apikey: 'Введите ваш TikTok API-ключ (бесплатный — на tik.tools):',
    tiktok_connect_error: 'Не удалось подключиться. Проверьте юзернейм и API-ключ (бесплатный можно получить на tik.tools), и убедитесь, что у вас идёт LIVE.',
    connect_twitch_title: 'Подключить Twitch',
    connect_twitch_subtitle: 'Мы будем читать чат, чтобы считать голоса.',
    connect_twitch_label: 'Название канала',
    connect_twitch_placeholder: 'your channel',
    connect_tiktok_title: 'Подключить TikTok',
    connect_tiktok_subtitle: 'Мы будем читать чат, чтобы считать голоса.',
    connect_tiktok_label_username: 'Юзернейм TikTok',
    connect_tiktok_placeholder_username: 'your username',
    connect_tiktok_label_apikey: 'API-ключ',
    connect_tiktok_placeholder_apikey: 'Вставьте ваш API-ключ',
    connect_tiktok_hint: 'Бесплатный ключ можно получить на <a href="https://tik.tools/login" target="_blank" rel="noopener">tik.tools</a> — займёт секунд 30.',
    connect_modal_confirm: 'Подключить',
    connect_modal_connecting: 'Подключение…',
    settings_panic_title: 'Паник-кнопка',
    settings_panic_hint: 'Мгновенно вырубает текущий скример — видео, звук, всё — независимо от того, что его запустило. Таймеры и голосование чата продолжают работать.',
    settings_panic_hotkey_label: 'Горячая клавиша',
    settings_panic_change: 'Изменить',
    settings_panic_cancel_btn: 'Отмена',
    settings_panic_recording: 'Нажми комбинацию…',
    settings_panic_vote_hint: 'Если голосование чата включено, новое голосование появится примерно через 5 секунд после паник-кнопки.',
    settings_panic_error_modifier: 'Выбери хотя бы один модификатор (Ctrl, Alt или Shift).',
    settings_panic_error_key: 'Эта клавиша не поддерживается — попробуй букву, цифру, F-клавишу или пробел.',
    settings_panic_error_taken: 'Эта комбинация уже занята другим приложением в системе.',
    update_modal_title: 'Вышла новая версия?',
    update_modal_text: (version) => `Привет, спасибо за использование SoS на протяжении уже месяца! Скорее всего, уже вышла новая версия этого приложения, в которой появились новые крутые функции. Рекомендуем проверить, не вышло ли обновление. Ваша текущая версия: ${version}`,
    update_modal_check_btn: 'Проверить',
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

  document.querySelectorAll('[data-i18n-title]').forEach((el) => {
    const key = el.dataset.i18nTitle;
    if (dict[key]) el.title = dict[key];
  });

  document.documentElement.lang = lang;
  document.getElementById('langSelect').value = lang;

  refreshConnectionTexts();
  if (widgetLinkInput && widgetLinkInput.value) {
    widgetLinkInput.value = widgetLinkInput.value.replace(/lang=\w+/, `lang=${lang}`);
  }
  refreshUpdateModalText();
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

function setupAccordion(headerId, panelId) {
  const header = document.getElementById(headerId);
  const panel = document.getElementById(panelId);
  header.addEventListener('click', () => {
    const isOpen = !panel.classList.contains('open');
    panel.classList.toggle('open', isOpen);
    header.classList.toggle('open', isOpen);
    header.setAttribute('aria-expanded', String(isOpen));
  });
}
setupAccordion('obsAccordionBtn', 'obsAccordionPanel');
setupAccordion('tiktokAccordionBtn', 'tiktokAccordionPanel');

const toggleScreamersEl = document.getElementById('toggleScreamers');
const intervalMinEl = document.getElementById('intervalMin');
const intervalMaxEl = document.getElementById('intervalMax');
const toggleChatVoteEl = document.getElementById('toggleChatVote');
const voteSecondsEl = document.getElementById('voteSeconds');
const widgetLinkRow = document.getElementById('widgetLinkRow');
const widgetLinkInput = document.getElementById('widgetLinkInput');
const copyWidgetLinkBtn = document.getElementById('copyWidgetLinkBtn');

function readIntervalRange() {
  let min = parseInt(intervalMinEl.value, 10);
  let max = parseInt(intervalMaxEl.value, 10);
  if (!Number.isFinite(min) || min < 1) min = 1;
  if (!Number.isFinite(max) || max < 1) max = 1;
  if (max < min) max = min;
  intervalMinEl.value = min;
  intervalMaxEl.value = max;
  return { min, max };
}

function readVoteSeconds() {
  let seconds = parseInt(voteSecondsEl.value, 10);
  if (!Number.isFinite(seconds) || seconds < 5) seconds = 5;
  voteSecondsEl.value = seconds;
  return seconds;
}

async function ensureWidgetLink() {
  try {
    const port = await invoke('ensure_widget_server');
    widgetLinkInput.value = `http://127.0.0.1.nip.io:${port}/?lang=${currentLang}`;
    widgetLinkRow.classList.add('visible');
  } catch (e) {
    console.error('Failed to start the widget server:', e);
  }
}

copyWidgetLinkBtn.addEventListener('click', async () => {
  try {
    await navigator.clipboard.writeText(widgetLinkInput.value);
    copyWidgetLinkBtn.classList.add('copied');
    setTimeout(() => copyWidgetLinkBtn.classList.remove('copied'), 1200);
  } catch (e) {
    console.error('Clipboard write failed:', e);
  }
});

async function syncAutoScares() {
  if (!toggleScreamersEl.checked) {
    try {
      await invoke('stop_random_scares');
    } catch (e) {
      console.error('Failed to stop auto scares:', e);
    }
    return;
  }

  const { min, max } = readIntervalRange();
  const chatVote = toggleChatVoteEl.checked;
  const voteSeconds = readVoteSeconds();
  try {
    await invoke('start_random_scares', {
      minMinutes: min,
      maxMinutes: max,
      chatVote,
      voteSeconds,
    });
  } catch (e) {
    console.error('Failed to start auto scares:', e);
  }
}

toggleScreamersEl.addEventListener('change', syncAutoScares);
intervalMinEl.addEventListener('change', syncAutoScares);
intervalMaxEl.addEventListener('change', syncAutoScares);
voteSecondsEl.addEventListener('change', syncAutoScares);

toggleChatVoteEl.addEventListener('change', () => {
  syncAutoScares();
  if (toggleChatVoteEl.checked) {
    ensureWidgetLink();
  }
});

const masterVolumeEl = document.getElementById('masterVolume');
const volumeValueEl = document.getElementById('volumeValue');

function paintVolumeSlider(percent) {
  volumeValueEl.textContent = `${percent}%`;
  masterVolumeEl.style.background =
    `linear-gradient(to right, var(--accent-color) 0%, var(--accent-color) ${percent}%, #2a2a2a ${percent}%, #2a2a2a 100%)`;
}

async function commitMasterVolume() {
  const percent = parseInt(masterVolumeEl.value, 10);
  localStorage.setItem('sos_volume', percent);
  try {
    await invoke('set_master_volume', { volume: percent / 100 });
  } catch (e) {
    console.error('Failed to set master volume:', e);
  }
}

masterVolumeEl.addEventListener('input', () => paintVolumeSlider(parseInt(masterVolumeEl.value, 10)));
masterVolumeEl.addEventListener('change', commitMasterVolume);

async function initMasterVolume() {
  const saved = parseInt(localStorage.getItem('sos_volume'), 10);
  const percent = Number.isFinite(saved) ? Math.min(100, Math.max(0, saved)) : 100;
  masterVolumeEl.value = percent;
  paintVolumeSlider(percent);
  try {
    await invoke('set_master_volume', { volume: percent / 100 });
  } catch (e) {
    console.error('Failed to init master volume:', e);
  }
}

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
let twitchChannel = '';
let tiktokUsername = '';

function refreshConnectionTexts() {
  const dict = translations[currentLang];
  const twitchBtn = document.getElementById('twitchBtn');
  const twitchStatus = document.getElementById('twitchStatus');
  const tiktokBtn = document.getElementById('tiktokBtn');
  const tiktokStatus = document.getElementById('tiktokStatus');

  twitchBtn.textContent = connectionState.twitch ? dict.settings_disconnect : dict.settings_connect;
  twitchBtn.classList.toggle('connected', connectionState.twitch);
  twitchStatus.textContent = connectionState.twitch
    ? `${dict.settings_connected}${twitchChannel ? ': ' + twitchChannel : ''}`
    : dict.settings_not_connected;
  twitchStatus.classList.toggle('connected', connectionState.twitch);

  tiktokBtn.textContent = connectionState.tiktok ? dict.settings_disconnect : dict.settings_connect;
  tiktokBtn.classList.toggle('connected', connectionState.tiktok);
  tiktokStatus.textContent = connectionState.tiktok
    ? `${dict.settings_connected}${tiktokUsername ? ': ' + tiktokUsername : ''}`
    : dict.settings_not_connected;
  tiktokStatus.classList.toggle('connected', connectionState.tiktok);
}

document.getElementById('twitchBtn').addEventListener('click', async () => {
  if (connectionState.twitch) {
    try {
      await invoke('disconnect_twitch_chat');
    } catch (e) {
      console.error('Failed to disconnect Twitch chat:', e);
    }
    connectionState.twitch = false;
    twitchChannel = '';
    refreshConnectionTexts();
    return;
  }

  openConnectModal('twitch');
});

document.getElementById('tiktokBtn').addEventListener('click', async () => {
  if (connectionState.tiktok) {
    try {
      await invoke('disconnect_tiktok_chat');
    } catch (e) {
      console.error('Failed to disconnect TikTok chat:', e);
    }
    connectionState.tiktok = false;
    tiktokUsername = '';
    refreshConnectionTexts();
    return;
  }

  openConnectModal('tiktok');
});

const connectModal = document.getElementById('connectModal');
const connectModalClose = document.getElementById('connectModalClose');
const connectModalIcon = document.getElementById('connectModalIcon');
const connectModalTitle = document.getElementById('connectModalTitle');
const connectModalSubtitle = document.getElementById('connectModalSubtitle');
const connectModalField2 = document.getElementById('connectModalField2');
const connectModalLabel1 = document.getElementById('connectModalLabel1');
const connectModalLabel2 = document.getElementById('connectModalLabel2');
const connectModalInput1 = document.getElementById('connectModalInput1');
const connectModalInput2 = document.getElementById('connectModalInput2');
const connectModalHint = document.getElementById('connectModalHint');
const connectModalError = document.getElementById('connectModalError');
const connectModalConfirm = document.getElementById('connectModalConfirm');

let activeConnectPlatform = null;

function openConnectModal(platform) {
  const dict = translations[currentLang];
  activeConnectPlatform = platform;

  connectModalInput1.value = '';
  connectModalInput2.value = '';
  connectModalError.classList.add('hidden');
  connectModalError.textContent = '';
  connectModalConfirm.disabled = false;
  connectModalConfirm.textContent = dict.connect_modal_confirm;

  if (platform === 'twitch') {
    connectModalIcon.textContent = 'TW';
    connectModalIcon.style.background = '#9146FF';
    connectModalTitle.textContent = dict.connect_twitch_title;
    connectModalSubtitle.textContent = dict.connect_twitch_subtitle;
    connectModalLabel1.textContent = dict.connect_twitch_label;
    connectModalInput1.placeholder = dict.connect_twitch_placeholder;
    connectModalField2.classList.add('hidden');
    connectModalHint.classList.add('hidden');
  } else {
    connectModalIcon.textContent = 'TT';
    connectModalIcon.style.background = '#000000';
    connectModalTitle.textContent = dict.connect_tiktok_title;
    connectModalSubtitle.textContent = dict.connect_tiktok_subtitle;
    connectModalLabel1.textContent = dict.connect_tiktok_label_username;
    connectModalInput1.placeholder = dict.connect_tiktok_placeholder_username;
    connectModalLabel2.textContent = dict.connect_tiktok_label_apikey;
    connectModalInput2.placeholder = dict.connect_tiktok_placeholder_apikey;
    connectModalField2.classList.remove('hidden');
    connectModalHint.innerHTML = dict.connect_tiktok_hint;
    connectModalHint.classList.remove('hidden');
  }

  connectModal.classList.add('visible');
  connectModalInput1.focus();
}

function closeConnectModal() {
  connectModal.classList.remove('visible');
  activeConnectPlatform = null;
}

async function submitConnectModal() {
  const dict = translations[currentLang];

  if (activeConnectPlatform === 'twitch') {
    const channel = connectModalInput1.value.trim().replace(/^[#@]+/, '');
    if (!channel) {
      connectModalInput1.focus();
      return;
    }

    connectModalConfirm.disabled = true;
    connectModalConfirm.textContent = dict.connect_modal_connecting;
    connectModalError.classList.add('hidden');

    try {
      await invoke('connect_twitch_chat', { channel });
      connectionState.twitch = true;
      twitchChannel = channel;
      refreshConnectionTexts();
      closeConnectModal();
      return;
    } catch (e) {
      console.error('Failed to connect to Twitch chat:', e);
      connectModalError.textContent = dict.twitch_connect_error;
      connectModalError.classList.remove('hidden');
    }
  } else if (activeConnectPlatform === 'tiktok') {
    const username = connectModalInput1.value.trim();
    const apiKey = connectModalInput2.value.trim();
    if (!username) {
      connectModalInput1.focus();
      return;
    }
    if (!apiKey) {
      connectModalInput2.focus();
      return;
    }

    connectModalConfirm.disabled = true;
    connectModalConfirm.textContent = dict.connect_modal_connecting;
    connectModalError.classList.add('hidden');

    try {
      await invoke('connect_tiktok_chat', { username, apiKey });
      connectionState.tiktok = true;
      tiktokUsername = username.replace(/^@/, '');
      refreshConnectionTexts();
      closeConnectModal();
      return;
    } catch (e) {
      console.error('Failed to connect to TikTok chat:', e);
      connectModalError.textContent = dict.tiktok_connect_error;
      connectModalError.classList.remove('hidden');
    }
  }

  connectModalConfirm.disabled = false;
  connectModalConfirm.textContent = dict.connect_modal_confirm;
}

connectModalConfirm.addEventListener('click', submitConnectModal);
connectModalClose.addEventListener('click', closeConnectModal);
connectModal.addEventListener('click', (e) => {
  if (e.target === connectModal) closeConnectModal();
});
[connectModalInput1, connectModalInput2].forEach((input) => {
  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') submitConnectModal();
    if (e.key === 'Escape') closeConnectModal();
  });
});

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

document.querySelectorAll('.donate-card').forEach((link) => {
  link.addEventListener('click', (e) => {
    e.preventDefault();
    const url = link.getAttribute('href');
    if (!url) return;
    openUrl(url).catch((err) => console.error('Failed to open donation link:', err));
  });
});

const APP_VERSION = 'v0.1.2';
const UPDATE_CHECK_URL = 'https://github.com/BLACKLINE-production/Scream-on-Stream';
const MS_PER_DAY = 24 * 60 * 60 * 1000;
const UPDATE_NOTICE_INTERVAL_DAYS = 30;

const updateModal = document.getElementById('updateModal');
const updateModalText = document.getElementById('updateModalText');
const updateModalClose = document.getElementById('updateModalClose');
const updateModalCheckBtn = document.getElementById('updateModalCheckBtn');

function refreshUpdateModalText() {
  if (updateModalText) {
    updateModalText.textContent = translations[currentLang].update_modal_text(APP_VERSION);
  }
}

function showUpdateModal() {
  refreshUpdateModalText();
  updateModal.classList.add('visible');
}

function closeUpdateModal() {
  updateModal.classList.remove('visible');
}

updateModalClose.addEventListener('click', closeUpdateModal);
updateModal.addEventListener('click', (e) => {
  if (e.target === updateModal) closeUpdateModal();
});
updateModalCheckBtn.addEventListener('click', () => {
  openUrl(UPDATE_CHECK_URL).catch((err) => console.error('Failed to open update-check link:', err));
  closeUpdateModal();
});

function initUpdateNotice() {
  const now = Date.now();

  let firstLaunch = parseInt(localStorage.getItem('sos_first_launch'), 10);
  if (!firstLaunch) {
    localStorage.setItem('sos_first_launch', String(now));
    return;
  }

  const daysSinceInstall = (now - firstLaunch) / MS_PER_DAY;
  if (daysSinceInstall < UPDATE_NOTICE_INTERVAL_DAYS) return;

  const lastShown = parseInt(localStorage.getItem('sos_update_notice_last_shown'), 10) || 0;
  const daysSinceLastShown = (now - lastShown) / MS_PER_DAY;
  if (lastShown && daysSinceLastShown < UPDATE_NOTICE_INTERVAL_DAYS) return;

  showUpdateModal();
  localStorage.setItem('sos_update_notice_last_shown', String(now));
}


const PANIC_DEFAULT_PAYLOAD = { code: 'KeyP', ctrl: true, alt: true, shift: false, meta: false };

const panicHotkeyBadge = document.getElementById('panicHotkeyBadge');
const panicHotkeyChangeBtn = document.getElementById('panicHotkeyChangeBtn');
const panicHotkeyError = document.getElementById('panicHotkeyError');

let recordingHotkey = false;
let currentHotkeyPayload = PANIC_DEFAULT_PAYLOAD;

function codeToDisplay(code) {
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  return code; 
}

function buildHotkeyDisplay({ code, ctrl, alt, shift, meta }) {
  const parts = [];
  if (ctrl) parts.push('Ctrl');
  if (alt) parts.push('Alt');
  if (shift) parts.push('Shift');
  if (meta) parts.push('Win');
  parts.push(codeToDisplay(code));
  return parts.join('+');
}

function isSupportedHotkeyCode(code) {
  return /^Key[A-Z]$/.test(code) || /^Digit[0-9]$/.test(code) || /^F([1-9]|1[0-2])$/.test(code) || code === 'Space';
}

function setPanicError(key) {
  panicHotkeyError.textContent = key ? translations[currentLang][key] : '';
}

function stopRecordingHotkey() {
  recordingHotkey = false;
  document.removeEventListener('keydown', onHotkeyKeydown, true);
  panicHotkeyBadge.classList.remove('recording');
  panicHotkeyBadge.textContent = buildHotkeyDisplay(currentHotkeyPayload);
  panicHotkeyChangeBtn.textContent = translations[currentLang].settings_panic_change;
}

async function onHotkeyKeydown(e) {
  e.preventDefault();
  e.stopPropagation();

  if (e.key === 'Escape') {
    stopRecordingHotkey();
    return;
  }
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
    return;
  }

  const payload = {
    code: e.code,
    ctrl: e.ctrlKey,
    alt: e.altKey,
    shift: e.shiftKey,
    meta: e.metaKey,
  };

  if (!payload.ctrl && !payload.alt && !payload.shift) {
    setPanicError('settings_panic_error_modifier');
    return;
  }
  if (!isSupportedHotkeyCode(payload.code)) {
    setPanicError('settings_panic_error_key');
    return;
  }

  try {
    await invoke('set_panic_hotkey', payload);
    setPanicError(null);
    currentHotkeyPayload = payload;
    localStorage.setItem('sos_panic_hotkey', JSON.stringify(payload));
  } catch (err) {
    console.error('Failed to set panic hotkey:', err);
    setPanicError('settings_panic_error_taken');
  }

  stopRecordingHotkey();
}

panicHotkeyChangeBtn.addEventListener('click', () => {
  if (recordingHotkey) {
    stopRecordingHotkey();
    return;
  }
  recordingHotkey = true;
  setPanicError(null);
  panicHotkeyBadge.classList.add('recording');
  panicHotkeyBadge.textContent = translations[currentLang].settings_panic_recording;
  panicHotkeyChangeBtn.textContent = translations[currentLang].settings_panic_cancel_btn;
  document.addEventListener('keydown', onHotkeyKeydown, true);
});

async function initPanicHotkey() {
  let saved = null;
  try {
    saved = JSON.parse(localStorage.getItem('sos_panic_hotkey'));
  } catch (e) {
    saved = null;
  }
  const payload = saved && saved.code ? saved : PANIC_DEFAULT_PAYLOAD;
  currentHotkeyPayload = payload;
  panicHotkeyBadge.textContent = buildHotkeyDisplay(payload);
  try {
    await invoke('set_panic_hotkey', payload);
  } catch (e) {
    console.error('Failed to apply saved panic hotkey, keeping default:', e);
    currentHotkeyPayload = PANIC_DEFAULT_PAYLOAD;
    panicHotkeyBadge.textContent = buildHotkeyDisplay(PANIC_DEFAULT_PAYLOAD);
  }
}

const toggleAutostartEl = document.getElementById('toggleAutostart');

toggleAutostartEl.addEventListener('change', async () => {
  const enabled = toggleAutostartEl.checked;
  try {
    await invoke('set_autostart', { enabled });
  } catch (e) {
    console.error('Failed to update autostart setting:', e);
    toggleAutostartEl.checked = !enabled;
  }
});

async function initAutostart() {
  try {
    toggleAutostartEl.checked = await invoke('get_autostart_enabled');
  } catch (e) {
    console.error('Failed to read autostart setting:', e);
  }
}

initLanguage();
loadScreamers();
initMasterVolume();
initPanicHotkey();
initAutostart();
initUpdateNotice();