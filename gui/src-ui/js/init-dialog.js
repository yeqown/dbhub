/**
 * Check initialization status and show dialog if needed
 */
async function checkAndShowInitDialog() {
    try {
        // Get init status from Tauri state
        const initResult = await invoke('get_init_status');

        if (initResult.status === 'NotInitialized' ||
            initResult.status === 'NoValidConfig') {

            // Show confirmation dialog
            const confirmed = confirm(
                'Welcome to DB Hub!\n\n' +
                'No configuration file found. Would you like to create a default configuration?\n\n' +
                'Configuration location: ~/.dbhub/config.yml'
            );

            if (confirmed) {
                try {
                    await invoke('initialize_config');
                    alert('Configuration file created successfully!');
                } catch (error) {
                    alert('Failed to create configuration file: ' + error);
                    // Exit on error
                    window.close();
                }
            } else {
                // User cancelled, exit application
                window.close();
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
