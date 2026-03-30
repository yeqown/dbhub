// Manage Page JavaScript

const { invoke } = window.__TAURI__.core;

// State
let configFiles = [];
let currentFile = null;
let originalContent = '';
let isModified = false;
let codeMirror = null;
let pendingDeleteFile = null;
let pendingSelectFile = null;

// DOM Elements
const fileList = document.getElementById('file-list');
const editorPlaceholder = document.getElementById('editor-placeholder');
const editorContainer = document.getElementById('editor-container');
const editorWrapper = document.getElementById('editor');
const currentFileName = document.getElementById('current-file-name');
const btnSave = document.getElementById('btn-save');
const btnCancel = document.getElementById('btn-cancel');
const btnAdd = document.getElementById('btn-add');
const editorActions = document.querySelector('.editor-actions');
const statusMessage = document.getElementById('status-message');
const currentConfigPath = document.getElementById('current-config-path');
const lastSaved = document.getElementById('last-saved');

// Modals
const addModal = document.getElementById('add-modal');
const newFileNameInput = document.getElementById('new-file-name');
const modalCancel = document.getElementById('modal-cancel');
const modalCreate = document.getElementById('modal-create');

const confirmModal = document.getElementById('confirm-modal');
const confirmTitle = document.getElementById('confirm-title');
const confirmMessage = document.getElementById('confirm-message');
const confirmCancelBtn = document.getElementById('confirm-cancel');
const confirmOkBtn = document.getElementById('confirm-ok');

const discardModal = document.getElementById('discard-modal');
const discardCancelBtn = document.getElementById('discard-cancel');
const discardOkBtn = document.getElementById('discard-ok');

// Initialize
document.addEventListener('DOMContentLoaded', init);

async function init() {
    initCodeMirror();
    await loadConfigFiles();
    setupEventListeners();
}

function initCodeMirror() {
    codeMirror = CodeMirror(editorWrapper, {
        value: '',
        mode: 'yaml',
        lineNumbers: true,
        lineWrapping: true,
        indentUnit: 2,
        tabSize: 2,
        indentWithTabs: false,
        autoCloseBrackets: true,
        viewportMargin: Infinity,
        readOnly: true,
    });

    codeMirror.on('change', () => {
        if (currentFile) {
            isModified = codeMirror.getValue() !== originalContent;
            updateButtonStates();
            updateModifiedIndicator();
        }
    });
}

// --- Custom confirm modals (Promise-based) ---

function showConfirmModal(title, message) {
    return new Promise((resolve) => {
        confirmTitle.textContent = title;
        confirmMessage.textContent = message;
        confirmModal.classList.add('show');

        function cleanup() {
            confirmModal.classList.remove('show');
            confirmCancelBtn.removeEventListener('click', onCancel);
            confirmOkBtn.removeEventListener('click', onOk);
            confirmModal.querySelector('.modal-backdrop').removeEventListener('click', onCancel);
        }

        function onCancel() { cleanup(); resolve(false); }
        function onOk() { cleanup(); resolve(true); }

        confirmCancelBtn.addEventListener('click', onCancel);
        confirmOkBtn.addEventListener('click', onOk);
        confirmModal.querySelector('.modal-backdrop').addEventListener('click', onCancel);
    });
}

function showDiscardModal() {
    return new Promise((resolve) => {
        discardModal.classList.add('show');

        function cleanup() {
            discardModal.classList.remove('show');
            discardCancelBtn.removeEventListener('click', onCancel);
            discardOkBtn.removeEventListener('click', onOk);
            discardModal.querySelector('.modal-backdrop').removeEventListener('click', onCancel);
        }

        function onCancel() { cleanup(); resolve(false); }
        function onOk() { cleanup(); resolve(true); }

        discardCancelBtn.addEventListener('click', onCancel);
        discardOkBtn.addEventListener('click', onOk);
        discardModal.querySelector('.modal-backdrop').addEventListener('click', onCancel);
    });
}

// --- Config file operations ---

async function loadConfigFiles() {
    try {
        configFiles = await invoke('get_config_files');
        renderFileList();
        setStatus('Ready', 'normal');
    } catch (error) {
        setStatus(`Error: ${error}`, 'error');
    }
}

function renderFileList() {
    fileList.innerHTML = '';

    if (configFiles.length === 0) {
        fileList.innerHTML = '<div class="empty-state"><p>No config files</p></div>';
        return;
    }

    configFiles.forEach((file, index) => {
        const item = document.createElement('div');
        item.className = 'file-item';
        if (currentFile && currentFile.path === file.path) {
            item.classList.add('active');
        }
        item.dataset.path = file.path;
        item.dataset.index = index;

        const nameSpan = document.createElement('span');
        nameSpan.className = 'file-item-name';
        nameSpan.textContent = file.name;

        const deleteBtn = document.createElement('button');
        deleteBtn.className = 'file-item-delete';
        deleteBtn.title = 'Delete';
        deleteBtn.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
            <line x1="10" y1="11" x2="10" y2="17"/>
            <line x1="14" y1="11" x2="14" y2="17"/>
        </svg>`;
        deleteBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            deleteFile(file);
        });

        item.appendChild(nameSpan);
        item.appendChild(deleteBtn);
        item.addEventListener('click', () => selectFile(file));

        fileList.appendChild(item);
    });
}

async function selectFile(file) {
    if (isModified) {
        const discard = await showDiscardModal();
        if (!discard) return;
    }

    currentFile = file;
    currentConfigPath.textContent = `Current Config Path: ${file.path}`;

    try {
        const content = await invoke('read_config_file', { path: file.path });
        originalContent = content;
        codeMirror.setValue(content);
        codeMirror.setOption('readOnly', false);
        isModified = false;

        editorPlaceholder.style.display = 'none';
        editorContainer.style.display = 'flex';
        currentFileName.textContent = file.name;

        updateButtonStates();
        renderFileList();
        setStatus('Loaded', 'success');

        setTimeout(() => codeMirror.refresh(), 10);
    } catch (error) {
        setStatus(`Error loading file: ${error}`, 'error');
    }
}

function setupEventListeners() {
    btnAdd.addEventListener('click', showAddModal);
    btnSave.addEventListener('click', saveFile);
    btnCancel.addEventListener('click', cancelChanges);

    modalCancel.addEventListener('click', hideAddModal);
    modalCreate.addEventListener('click', createNewFile);
    addModal.querySelector('.modal-backdrop').addEventListener('click', hideAddModal);

    newFileNameInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') createNewFile();
        else if (e.key === 'Escape') hideAddModal();
    });

    document.addEventListener('keydown', (e) => {
        if ((e.metaKey || e.ctrlKey) && e.key === 's') {
            e.preventDefault();
            if (!btnSave.disabled) saveFile();
        }
    });
}

function updateButtonStates() {
    btnSave.disabled = !isModified;
    btnCancel.disabled = !isModified;

    if (editorActions) {
        editorActions.classList.toggle('visible', isModified);
    }
}

function updateModifiedIndicator() {
    currentFileName.classList.toggle('modified', isModified);
}

async function saveFile() {
    if (!currentFile || !isModified) return;

    try {
        const content = codeMirror.getValue();
        await invoke('save_config_file', {
            path: currentFile.path,
            content: content
        });
        originalContent = content;
        isModified = false;
        updateButtonStates();
        updateModifiedIndicator();

        const now = new Date();
        lastSaved.textContent = `Saved at ${now.toLocaleTimeString()}`;
        setStatus('Saved', 'success');
    } catch (error) {
        setStatus(`Error saving: ${error}`, 'error');
    }
}

function cancelChanges() {
    if (!isModified) return;

    codeMirror.setValue(originalContent);
    isModified = false;
    updateButtonStates();
    updateModifiedIndicator();
    setStatus('Changes discarded', 'normal');
}

function showAddModal() {
    addModal.classList.add('show');
    newFileNameInput.value = '';
    newFileNameInput.focus();
}

function hideAddModal() {
    addModal.classList.remove('show');
}

async function createNewFile() {
    const name = newFileNameInput.value.trim();
    if (!name) {
        setStatus('Please enter a file name', 'error');
        return;
    }

    try {
        const newPath = await invoke('create_config_file', { name });
        hideAddModal();
        await loadConfigFiles();

        const newFile = configFiles.find(f => f.path === newPath);
        if (newFile) selectFile(newFile);

        setStatus('Config file created', 'success');
    } catch (error) {
        setStatus(`Error: ${error}`, 'error');
    }
}

async function deleteFile(file) {
    if (configFiles.length <= 1) {
        setStatus('Cannot delete the last config file', 'error');
        return;
    }

    const confirmed = await showConfirmModal(
        'Delete Config File',
        `Are you sure you want to delete "${file.name}"?\n\nThis action cannot be undone.`
    );
    if (!confirmed) return;

    try {
        await invoke('delete_config_file', { path: file.path });

        if (currentFile && currentFile.path === file.path) {
            currentFile = null;
            originalContent = '';
            isModified = false;
            codeMirror.setValue('');
            codeMirror.setOption('readOnly', true);
            editorPlaceholder.style.display = 'flex';
            editorContainer.style.display = 'none';
        }

        await loadConfigFiles();
        setStatus('Config file deleted', 'success');
    } catch (error) {
        setStatus(`Error: ${error}`, 'error');
    }
}

function setStatus(message, type = 'normal') {
    statusMessage.textContent = message;
    statusMessage.className = 'status-message';

    if (type === 'error') {
        statusMessage.classList.add('error');
    } else if (type === 'success') {
        statusMessage.classList.add('success');
    }

    if (type === 'success') {
        setTimeout(() => {
            if (statusMessage.textContent === message) {
                statusMessage.textContent = 'Ready';
                statusMessage.className = 'status-message';
            }
        }, 3000);
    }
}
