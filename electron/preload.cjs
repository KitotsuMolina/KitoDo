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
  }
});
