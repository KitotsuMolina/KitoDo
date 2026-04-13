const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('kitodo', {
  invoke(command, payload = {}) {
    return ipcRenderer.invoke('kitodo:invoke', command, payload);
  },
  window: {
    isMaximized() {
      return ipcRenderer.invoke('kitodo:window:isMaximized');
    },
    toggleMaximize() {
      return ipcRenderer.invoke('kitodo:window:toggleMaximize');
    },
    close() {
      return ipcRenderer.invoke('kitodo:window:close');
    }
  },
  shell: {
    openExternal(url) {
      return ipcRenderer.invoke('kitodo:shell:openExternal', url);
    }
  },
  notifications: {
    getSettings() {
      return ipcRenderer.invoke('kitodo:notifications:getSettings');
    },
    setEnabled(enabled) {
      return ipcRenderer.invoke('kitodo:notifications:setEnabled', enabled);
    }
  }
});
