import { invoke } from '@tauri-apps/api/tauri';

let config = null;

async function loadConfig() {
    try {
        config = await invoke('get_config');
        renderDatabases();
        renderTemplates();
    } catch (error) {
        console.error('Failed to load config:', error);
        alert('Failed to load config: ' + error);
    }
}

function renderDatabases() {
    const container = document.getElementById('databases-list');
    container.innerHTML = '';

    config.databases.forEach((db, index) => {
        const div = document.createElement('div');
        div.style.marginBottom = '15px';
        div.style.padding = '10px';
        div.style.border = '1px solid var(--border-color)';
        div.style.borderRadius = '4px';
        div.innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <strong>${escapeHtml(db.alias)}</strong>
                <small>${escapeHtml(db.db_type)} - ${escapeHtml(db.env)}</small>
            </div>
            <div style="font-size: 12px; color: var(--text-secondary); margin-top: 5px;">
                ${escapeHtml(db.dsn)}
            </div>
        `;
        container.appendChild(div);
    });
}

function renderTemplates() {
    const container = document.getElementById('templates-list');
    container.innerHTML = '';

    if (!config.templates || Object.keys(config.templates).length === 0) {
        container.innerHTML = '<p style="color: var(--text-secondary);">No templates defined</p>';
        return;
    }

    for (const [name, template] of Object.entries(config.templates)) {
        const div = document.createElement('div');
        div.style.marginBottom = '10px';
        div.style.padding = '10px';
        div.style.border = '1px solid var(--border-color)';
        div.style.borderRadius = '4px';
        div.innerHTML = `
            <strong>${escapeHtml(name)}</strong>
            <div style="font-family: monospace; margin-top: 5px; color: var(--text-secondary);">
                ${escapeHtml(template.dsn)}
            </div>
        `;
        container.appendChild(div);
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

async function saveConfig() {
    try {
        await invoke('save_config_dto', { config });
        alert('Config saved successfully');
        // Close window
        if (window.__TAURI__) {
            const currentWindow = window.__TAURI__.window.getCurrent();
            currentWindow.close();
        }
    } catch (error) {
        alert('Failed to save config: ' + error);
    }
}

function cancel() {
    if (window.__TAURI__) {
        const currentWindow = window.__TAURI__.window.getCurrent();
        currentWindow.close();
    }
}

// Event listeners
document.getElementById('save-btn').addEventListener('click', saveConfig);
document.getElementById('cancel-btn').addEventListener('click', cancel);

// Initial load
loadConfig();
