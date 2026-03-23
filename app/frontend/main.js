const { app, BrowserWindow } = require('electron');
const path = require('path');
const { spawn } = require('child_process');

let pyProc = null;

function startPythonServer() {
    const script = path.join(__dirname, '../backend/dist/server/server.exe');
    pyProc = spawn(script);

    pyProc.stdout.on('data', (data) => {
        console.log(`PYTHON: ${data}`);
    });

    pyProc.stderr.on('data', (data) => {
        console.error(`PYTHON ERROR: ${data}`);
    });

    pyProc.on('close', () => {
        console.log('Python server stopped.');
    });
}

function createWindow() {
    const win = new BrowserWindow({
        width: 800,
        height: 600,
        webPreferences: { nodeIntegration: true }
    });
    win.loadFile('index.html');
}

app.whenReady().then(() => {
    startPythonServer();
    createWindow();
});

app.on('will-quit', () => {
    if (pyProc) pyProc.kill();
});