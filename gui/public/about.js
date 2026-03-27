// Handle repository link click
document.getElementById('repo-link').addEventListener('click', function(e) {
    e.preventDefault();
    const repoUrl = 'https://github.com/yeqown/dbhub';

    // Invoke Tauri command to open URL in default browser
    if (window.__TAURI__) {
        window.__TAURI__.invoke('open_repository', { url: repoUrl });
    }
});
