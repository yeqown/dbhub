// Initialize app
console.log('[main] DB Hub GUI starting...');

// Hide main window on startup
window.addEventListener('DOMContentLoaded', () => {
    console.log('[main] DOMContentLoaded fired');
    const currentWindow = window.__TAURI__.window.getCurrent();
    currentWindow.hide();
    console.log('[main] Window hidden');
});
