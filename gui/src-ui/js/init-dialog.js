/**
 * Check initialization status and show dialog if needed
 */
async function checkAndShowInitDialog() {
    try {
        // Get init status from Tauri state
        const initResult = await invoke('get_init_status');

        if (initResult.status === 'NotInitialized' ||
            initResult.status === 'NoValidConfig') {

            // Show the window first (it's hidden by default in menubar apps)
            const currentWindow = window.__TAURI__.window.getCurrent();
            await currentWindow.show();

            // Now use browser confirm/alert since window is visible
            const confirmed = confirm(
                'Welcome to DB Hub!\n\n' +
                'No configuration file found. Would you like to create a default configuration?\n\n' +
                'Configuration location: ~/.dbhub/config.yml'
            );

            if (confirmed) {
                try {
                    await invoke('initialize_config');
                    alert('Configuration file created successfully!');

                    // Hide window after successful initialization
                    await currentWindow.hide();
                } catch (error) {
                    alert('Failed to create configuration file: ' + error);
                    // Exit on error - close the app
                    const app = window.__TAURI__.app;
                    await app.exit(1);
                }
            } else {
                // User cancelled, exit application
                const app = window.__TAURI__.app;
                await app.exit(0);
            }
        }
    } catch (error) {
        console.error('Failed to check init status:', error);
    }
}

// Call on app startup
window.addEventListener('DOMContentLoaded', () => {
    checkAndShowInitDialog();
});
