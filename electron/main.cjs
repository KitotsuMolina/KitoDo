const { app, BrowserWindow, ipcMain, shell } = require('electron');
const { spawn } = require('node:child_process');
const { createServer } = require('node:http');
const { readFile } = require('node:fs/promises');
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

  const data = await response.json();
  if (!response.ok) {
    throw new Error(data?.error || `Falló la invocación ${command}`);
  }

  return data;
}

async function createWindow() {
  await startSidecar();
  await startStaticServer();

  mainWindow = new BrowserWindow({
    width: 1180,
    height: 780,
    minWidth: 980,
    minHeight: 640,
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

  if (isDev) {
    await mainWindow.loadURL(devServerUrl);
  } else {
    await mainWindow.loadURL(staticBaseUrl);
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

    app.on('activate', async () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        await createWindow();
      }
    });
  })
  .catch((error) => {
    console.error('Fallo iniciando KitoDo:', error);
    app.quit();
  });

process.on('unhandledRejection', (error) => {
  console.error('Unhandled rejection en main:', error);
});
