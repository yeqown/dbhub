import { app, Menu } from '@tauri-apps/api';

async function createMenu() {
    try {
        const connections = await invoke('get_connections');

        // Build Connect submenu
        const connectMenuItems = [];

        for (const [env, databases] of Object.entries(connections)) {
            const envItems = databases.map(db => ({
                label: db.alias,
                click: () => invoke('connect', { alias: db.alias })
            }));

            connectMenuItems.push({
                label: env,
                submenu: envItems
            });
        }

        // Main menu (Tauri uses a different format)
        // For now, we'll use simple status bar menu
        console.log('Menu created with', Object.keys(connections).length, 'environments');

    } catch (error) {
        console.error('Failed to create menu:', error);
    }
}

// Initialize menu on app ready
app.onReady(() => {
    createMenu();
    console.log('App initialized');
});

export { createMenu };
