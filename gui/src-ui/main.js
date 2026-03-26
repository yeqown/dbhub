// Initialize app
console.log('DB Hub GUI starting...');

// Hide main window on startup
window.addEventListener('DOMContentLoaded', () => {
    const currentWindow = window.__TAURI__.window.getCurrent();
    currentWindow.hide();
});
