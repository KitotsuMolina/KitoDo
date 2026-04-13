const { app, BrowserWindow, ipcMain, Notification, shell } = require('electron');
const { spawn } = require('node:child_process');
const { createServer } = require('node:http');
const { mkdir, readFile, writeFile } = require('node:fs/promises');
const net = require('node:net');
const path = require('node:path');
const process = require('node:process');

const isDev = !app.isPackaged;
const devServerUrl = process.env.KITODO_RENDERER_URL || 'http://127.0.0.1:5173';
const serverHost = '127.0.0.1';

let mainWindow = null;
let sidecarProcess = null;
let apiBaseUrl = null;
let staticServer = null;
let staticBaseUrl = null;
let isQuitting = false;
let dueNotificationTimer = null;
let dueNotificationState = { enabled: true, delivered: {} };
let dueNotificationStateLoaded = false;
let dueNotificationCheckInFlight = false;

const DUE_NOTIFICATION_POLL_MS = 5 * 60 * 1000;
const DUE_NOTIFICATION_RETENTION_DAYS = 21;

app.setName('KitoDo');

function getAppSourcePath() {
  if (app.isPackaged) {
    return app.getAppPath();
  }

  return path.resolve(__dirname, '..');
}

function getRuntimeCwd() {
  if (app.isPackaged) {
    return process.resourcesPath;
  }

  return getAppSourcePath();
}

function getSidecarBinaryPath() {
  if (app.isPackaged) {
    return path.join(process.resourcesPath, 'bin', 'kitodo-server');
  }

  return path.join(getAppSourcePath(), 'src-tauri', 'target', 'debug', 'kitodo-server');
}

function reservePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.on('error', reject);
    server.listen(0, serverHost, () => {
      const address = server.address();
      if (!address || typeof address === 'string') {
        server.close(() => reject(new Error('No se pudo reservar un puerto local.')));
        return;
      }

      const { port } = address;
      server.close((closeError) => {
        if (closeError) {
          reject(closeError);
          return;
        }

        resolve(port);
      });
    });
  });
}

function getBuildPath() {
  return path.join(getAppSourcePath(), 'build');
}

function getContentType(filePath) {
  switch (path.extname(filePath)) {
    case '.html':
      return 'text/html; charset=utf-8';
    case '.js':
      return 'text/javascript; charset=utf-8';
    case '.css':
      return 'text/css; charset=utf-8';
    case '.json':
      return 'application/json; charset=utf-8';
    case '.svg':
      return 'image/svg+xml';
    case '.png':
      return 'image/png';
    case '.ico':
      return 'image/x-icon';
    default:
      return 'application/octet-stream';
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function parseJsonSafe(raw) {
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

function formatLocalYmd(date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

function shiftLocalDate(date, days) {
  const next = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  next.setDate(next.getDate() + days);
  return next;
}

function getDueNotificationStatePath() {
  return path.join(app.getPath('userData'), 'due-notifications.json');
}

async function loadDueNotificationState() {
  if (dueNotificationStateLoaded) {
    return dueNotificationState;
  }

  try {
    const raw = await readFile(getDueNotificationStatePath(), 'utf8');
    const parsed = parseJsonSafe(raw);
    dueNotificationState =
      parsed && typeof parsed === 'object'
        ? {
            enabled: parsed.enabled !== false,
            delivered: parsed.delivered && typeof parsed.delivered === 'object' ? parsed.delivered : {}
          }
        : { enabled: true, delivered: {} };
  } catch (error) {
    if (error && error.code !== 'ENOENT') {
      console.error('No se pudo cargar el estado de notificaciones de vencimiento:', error);
    }
    dueNotificationState = { enabled: true, delivered: {} };
  }

  dueNotificationStateLoaded = true;
  pruneDueNotificationState();
  return dueNotificationState;
}

function pruneDueNotificationState() {
  const cutoff = formatLocalYmd(shiftLocalDate(new Date(), -DUE_NOTIFICATION_RETENTION_DAYS));
  const nextDelivered = {};

  for (const [key, value] of Object.entries(dueNotificationState.delivered || {})) {
    if (typeof value === 'string' && value >= cutoff) {
      nextDelivered[key] = value;
    }
  }

  dueNotificationState.delivered = nextDelivered;
}

function getDueNotificationSettings() {
  return {
    enabled: dueNotificationState.enabled !== false,
    supported: Notification.isSupported()
  };
}

async function saveDueNotificationState() {
  try {
    await mkdir(path.dirname(getDueNotificationStatePath()), { recursive: true });
    await writeFile(getDueNotificationStatePath(), JSON.stringify(dueNotificationState, null, 2), 'utf8');
  } catch (error) {
    console.error('No se pudo guardar el estado de notificaciones de vencimiento:', error);
  }
}

async function focusOrCreateMainWindow() {
  if (!mainWindow || mainWindow.isDestroyed()) {
    await createWindow();
    return;
  }

  if (mainWindow.isMinimized()) {
    mainWindow.restore();
  }

  if (!mainWindow.isVisible()) {
    mainWindow.show();
  }

  mainWindow.focus();
}

function buildDueNotificationPayload(task, kind) {
  const projectSuffix = task.projectName ? ` · @${task.projectName}` : '';
  const dueSuffix = task.dueDate ? ` · ${task.dueDate}` : '';

  if (kind === 'overdue') {
    return {
      title: 'Tarea vencida en KitoDo',
      body: `${task.title}${projectSuffix}${dueSuffix}`
    };
  }

  return {
    title: 'Tarea para hoy en KitoDo',
    body: `${task.title}${projectSuffix}${dueSuffix}`
  };
}

async function collectDueNotificationCandidates() {
  const [overdueTasks, todayTasks] = await Promise.all([
    invoke('list_overdue', { showDone: false }),
    invoke('list_today', { showDone: false })
  ]);

  const todayKey = formatLocalYmd(new Date());
  const candidates = [];

  for (const task of overdueTasks) {
    if (!task?.id || task.status !== 'todo' || !task.dueDate) {
      continue;
    }

    candidates.push({
      key: `overdue:${task.id}:${todayKey}`,
      task,
      kind: 'overdue'
    });
  }

  for (const task of todayTasks) {
    if (!task?.id || task.status !== 'todo' || !task.dueDate) {
      continue;
    }

    candidates.push({
      key: `today:${task.id}:${task.dueDate}`,
      task,
      kind: 'today'
    });
  }

  return candidates;
}

async function runDueNotificationCheck() {
  if (dueNotificationCheckInFlight || !Notification.isSupported() || !apiBaseUrl) {
    return;
  }

  dueNotificationCheckInFlight = true;

  try {
    await loadDueNotificationState();
    if (dueNotificationState.enabled === false) {
      return;
    }
    const candidates = await collectDueNotificationCandidates();
    let stateChanged = false;
    const deliveredToday = formatLocalYmd(new Date());

    for (const candidate of candidates) {
      if (dueNotificationState.delivered[candidate.key]) {
        continue;
      }

      const payload = buildDueNotificationPayload(candidate.task, candidate.kind);
      const notification = new Notification({
        title: payload.title,
        body: payload.body,
        urgency: candidate.kind === 'overdue' ? 'critical' : 'normal'
      });

      notification.on('click', () => {
        focusOrCreateMainWindow().catch((error) => {
          console.error('No se pudo enfocar la ventana tras una notificación:', error);
        });
      });

      notification.show();
      dueNotificationState.delivered[candidate.key] = deliveredToday;
      stateChanged = true;
    }

    if (stateChanged) {
      pruneDueNotificationState();
      await saveDueNotificationState();
    }
  } catch (error) {
    console.error('Fallo comprobando notificaciones de tareas con fecha:', error);
  } finally {
    dueNotificationCheckInFlight = false;
  }
}

async function startDueNotificationScheduler() {
  if (dueNotificationTimer || !Notification.isSupported()) {
    return;
  }

  await loadDueNotificationState();
  if (dueNotificationState.enabled === false) {
    return;
  }
  await runDueNotificationCheck();
  dueNotificationTimer = setInterval(() => {
    runDueNotificationCheck().catch((error) => {
      console.error('Fallo en el scheduler de notificaciones de vencimiento:', error);
    });
  }, DUE_NOTIFICATION_POLL_MS);
}

function stopDueNotificationScheduler() {
  if (!dueNotificationTimer) {
    return;
  }

  clearInterval(dueNotificationTimer);
  dueNotificationTimer = null;
}

async function setDueNotificationsEnabled(enabled) {
  await loadDueNotificationState();
  dueNotificationState.enabled = Boolean(enabled);
  await saveDueNotificationState();

  if (!Notification.isSupported()) {
    return getDueNotificationSettings();
  }

  if (dueNotificationState.enabled) {
    await startDueNotificationScheduler();
    await runDueNotificationCheck();
  } else {
    stopDueNotificationScheduler();
  }

  return getDueNotificationSettings();
}

async function startStaticServer() {
  if (isDev || staticServer) {
    return;
  }

  const port = await reservePort();
  const buildRoot = getBuildPath();

  staticServer = createServer(async (request, response) => {
    try {
      const requestPath = decodeURIComponent((request.url || '/').split('?')[0]);
      const normalizedPath = requestPath === '/' ? 'index.html' : requestPath.replace(/^\/+/, '');
      const resolvedPath = path.resolve(buildRoot, normalizedPath);

      if (!resolvedPath.startsWith(buildRoot)) {
        response.writeHead(403).end('Forbidden');
        return;
      }

      let filePath = resolvedPath;

      try {
        const body = await readFile(filePath);
        response.writeHead(200, { 'content-type': getContentType(filePath) });
        response.end(body);
      } catch (error) {
        if (normalizedPath !== 'index.html') {
          filePath = path.join(buildRoot, 'index.html');
          const body = await readFile(filePath);
          response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
          response.end(body);
          return;
        }

        throw error;
      }
    } catch (error) {
      response.writeHead(500, { 'content-type': 'text/plain; charset=utf-8' });
      response.end(`Static server error: ${error.message}`);
    }
  });

  await new Promise((resolve, reject) => {
    staticServer.once('error', reject);
    staticServer.listen(port, serverHost, () => resolve());
  });

  staticBaseUrl = `http://${serverHost}:${port}`;
}

async function waitForServer(url, attempts = 60, delayMs = 250) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const response = await fetch(`${url}/health`);
      if (response.ok) {
        return;
      }
    } catch {}

    await new Promise((resolve) => setTimeout(resolve, delayMs));
  }

  throw new Error('El sidecar Rust no respondió a tiempo.');
}

async function waitForRenderer(url, attempts = 60, delayMs = 250) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const response = await fetch(url, { method: 'GET' });
      if (response.ok) {
        return;
      }
    } catch {}

    await sleep(delayMs);
  }

  throw new Error(`El renderer no respondió a tiempo en ${url}.`);
}

async function startSidecar() {
  if (sidecarProcess) {
    return;
  }

  const port = await reservePort();
  const binaryPath = getSidecarBinaryPath();

  sidecarProcess = spawn(binaryPath, ['--host', serverHost, '--port', String(port)], {
    cwd: getRuntimeCwd(),
    stdio: 'inherit',
    env: {
      ...process.env,
      RUST_LOG: process.env.RUST_LOG || 'info'
    }
  });

  sidecarProcess.once('exit', (code, signal) => {
    sidecarProcess = null;

    if (!isQuitting) {
      console.error(`kitodo-server terminó inesperadamente (${signal || code}).`);
      app.quit();
    }
  });

  sidecarProcess.once('error', (error) => {
    console.error(`No se pudo iniciar kitodo-server: ${error.message}`);
  });

  apiBaseUrl = `http://${serverHost}:${port}`;
  await waitForServer(apiBaseUrl);
}

async function invoke(command, payload = {}) {
  if (!apiBaseUrl) {
    throw new Error('El backend local no está inicializado.');
  }

  const response = await fetch(`${apiBaseUrl}/invoke/${encodeURIComponent(command)}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify(payload ?? {})
  });

  const raw = await response.text();
  const data = parseJsonSafe(raw);
  if (!response.ok) {
    throw new Error(data?.error || raw || `Falló la invocación ${command}`);
  }

  return data;
}

async function loadWindowContent(win, url) {
  if (isDev) {
    await waitForRenderer(url, 80, 250);
  }

  await win.loadURL(url);
}

async function createWindow() {
  await startSidecar();
  await startStaticServer();

  mainWindow = new BrowserWindow({
    width: 1180,
    height: 780,
    minWidth: 640,
    minHeight: 560,
    show: false,
    autoHideMenuBar: true,
    backgroundColor: '#101716',
    title: 'KitoDo',
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      sandbox: true,
      nodeIntegration: false,
      spellcheck: false,
      devTools: isDev
    }
  });

  mainWindow.once('ready-to-show', () => {
    mainWindow?.show();
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: 'deny' };
  });

  mainWindow.webContents.on('will-navigate', (event, url) => {
    if (isDev && url.startsWith(devServerUrl)) {
      return;
    }

    if (!isDev && staticBaseUrl && url.startsWith(staticBaseUrl)) {
      return;
    }

    event.preventDefault();
    shell.openExternal(url);
  });

  mainWindow.webContents.on('render-process-gone', (_event, details) => {
    if (!isQuitting) {
      console.error(`El renderer terminó inesperadamente (${details.reason}).`);
    }
  });

  if (isDev) {
    await loadWindowContent(mainWindow, devServerUrl);
  } else {
    await loadWindowContent(mainWindow, staticBaseUrl);
  }
}

ipcMain.handle('kitodo:invoke', async (_event, command, payload) => invoke(command, payload));

ipcMain.handle('kitodo:window:isMaximized', (event) => {
  const win = BrowserWindow.fromWebContents(event.sender);
  return win ? win.isMaximized() : false;
});

ipcMain.handle('kitodo:window:toggleMaximize', (event) => {
  const win = BrowserWindow.fromWebContents(event.sender);
  if (!win) {
    return false;
  }

  if (win.isMaximized()) {
    win.unmaximize();
    return false;
  }

  win.maximize();
  return true;
});

ipcMain.handle('kitodo:window:close', (event) => {
  const win = BrowserWindow.fromWebContents(event.sender);
  win?.close();
});

ipcMain.handle('kitodo:shell:openExternal', async (_event, url) => {
  if (typeof url !== 'string' || !/^https?:\/\//.test(url)) {
    throw new Error('URL externa inválida.');
  }

  await shell.openExternal(url);
});

ipcMain.handle('kitodo:notifications:getSettings', async () => {
  await loadDueNotificationState();
  return getDueNotificationSettings();
});

ipcMain.handle('kitodo:notifications:setEnabled', async (_event, enabled) => {
  return setDueNotificationsEnabled(enabled);
});

app.on('second-instance', () => {
  if (!mainWindow) {
    return;
  }

  if (mainWindow.isMinimized()) {
    mainWindow.restore();
  }
  mainWindow.focus();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('before-quit', () => {
  isQuitting = true;
  stopDueNotificationScheduler();
  if (sidecarProcess) {
    sidecarProcess.kill('SIGTERM');
  }
  if (staticServer) {
    staticServer.close();
  }
});

app.whenReady()
  .then(async () => {
    if (!app.requestSingleInstanceLock()) {
      app.quit();
      return;
    }

    await createWindow();
    await startDueNotificationScheduler();

    app.on('activate', async () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        await createWindow();
      }

      await runDueNotificationCheck();
    });
  })
  .catch((error) => {
    console.error('Fallo iniciando KitoDo:', error);
    app.quit();
  });

process.on('unhandledRejection', (error) => {
  console.error('Unhandled rejection en main:', error);
});

process.on('uncaughtException', (error) => {
  console.error('Excepción no controlada en main:', error);
});
