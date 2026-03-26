import { invoke } from '@tauri-apps/api/tauri';
import { dialog } from '@tauri-apps/api';

let allConnections = {};

async function loadContexts() {
    try {
        allConnections = await invoke('get_connections');
        renderTable(allConnections);
    } catch (error) {
        console.error('Failed to load contexts:', error);
        alert('Failed to load contexts: ' + error);
    }
}

function renderTable(connections) {
    const tbody = document.getElementById('context-list');
    tbody.innerHTML = '';

    for (const [env, databases] of Object.entries(connections)) {
        for (const db of databases) {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${escapeHtml(db.alias)}</td>
                <td>${escapeHtml(db.db_type)}</td>
                <td>${escapeHtml(db.env)}</td>
                <td>${escapeHtml(db.description || '-')}</td>
                <td>
                    <button class="btn-small" onclick="editContext('${escapeHtml(db.alias)}')">Edit</button>
                    <button class="btn-small btn-danger" onclick="deleteContext('${escapeHtml(db.alias)}')">Delete</button>
                </td>
            `;
            tbody.appendChild(row);
        }
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

async function addContext() {
    const alias = prompt('Alias:');
    if (!alias) return;

    const dbType = prompt('Database Type (mysql/mongo/redis):', '');
    if (!dbType) return;

    const dsn = prompt('DSN:', '');
    if (!dsn) return;

    const env = prompt('Environment:', '');
    if (!env) return;

    const description = prompt('Description:', '');

    try {
        await invoke('add_database', { db: {
            alias,
            db_type: dbType,
            dsn,
            env,
            description: description || null
        }});
        await loadContexts();
    } catch (error) {
        alert('Failed to add context: ' + error);
    }
}

async function editContext(alias) {
    alert('Edit functionality - to be implemented');
}

async function deleteContext(alias) {
    const confirmed = confirm(`Delete context "${alias}"?`);
    if (!confirmed) return;

    try {
        await invoke('delete_database', { alias });
        await loadContexts();
    } catch (error) {
        alert('Failed to delete context: ' + error);
    }
}

// Event listeners
document.getElementById('add-btn').addEventListener('click', addContext);

// Initial load
loadContexts();
