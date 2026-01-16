// CodexBar Tauri Frontend

const { invoke } = window.__TAURI__.core;

// State
let isRefreshing = false;

// DOM Elements
const providersContainer = document.getElementById('providers');
const lastUpdatedSpan = document.getElementById('lastUpdated');
const refreshBtn = document.getElementById('refreshBtn');
const settingsBtn = document.getElementById('settingsBtn');
const aboutBtn = document.getElementById('aboutBtn');
const cookiesBtn = document.getElementById('cookiesBtn');
const quitBtn = document.getElementById('quitBtn');

// Color mapping based on percentage (lower usage = better)
function getColorClass(percent) {
    if (percent === null || percent === undefined) return 'gray';
    if (percent <= 25) return 'green';   // 0-25% used = plenty remaining
    if (percent <= 50) return 'yellow';  // 25-50% used
    if (percent <= 75) return 'orange';  // 50-75% used
    return 'red';                         // 75-100% used = running low
}

// Format relative time
function formatRelativeTime(timestamp) {
    if (!timestamp) return 'Never';

    const now = Date.now();
    const diff = now - timestamp;

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return `${Math.floor(diff / 86400000)}d ago`;
}

// Render a single provider
function renderProvider(provider) {
    const colorClass = getColorClass(provider.percent);
    const percentText = provider.percent !== null ? `${provider.percent}%` : '--';

    let metaText = '';
    if (provider.used !== null && provider.limit !== null) {
        metaText = `${provider.used} / ${provider.limit} ${provider.unit || 'requests'}`;
        if (provider.reset_time) {
            metaText += ` • Resets ${provider.reset_time}`;
        }
    }

    const errorHtml = provider.error
        ? `<div class="provider-error" title="${provider.error}">${provider.error}</div>`
        : '';

    const metaHtml = metaText
        ? `<div class="provider-meta">${metaText}</div>`
        : '';

    return `
        <div class="provider" data-name="${provider.name}">
            <div class="provider-header">
                <span class="provider-name">
                    <span class="provider-status ${colorClass}"></span>
                    ${provider.name}
                </span>
                <span class="provider-percent ${colorClass}">${percentText}</span>
            </div>
            <div class="progress-bar">
                <div class="progress-fill ${colorClass}" style="width: ${provider.percent || 0}%"></div>
            </div>
            ${errorHtml}
            ${metaHtml}
        </div>
    `;
}

// Render all providers
function renderProviders(providers) {
    if (!providers || providers.length === 0) {
        providersContainer.innerHTML = '<div class="loading">No providers configured</div>';
        return;
    }

    providersContainer.innerHTML = providers.map(renderProvider).join('');
}

// Fetch and update provider data
async function refreshProviders() {
    if (isRefreshing) return;

    isRefreshing = true;
    refreshBtn.classList.add('spinning');

    try {
        const providers = await invoke('get_providers');
        renderProviders(providers);
        lastUpdatedSpan.textContent = 'Updated just now';
    } catch (error) {
        console.error('Failed to fetch providers:', error);
        lastUpdatedSpan.textContent = 'Update failed';
    } finally {
        isRefreshing = false;
        refreshBtn.classList.remove('spinning');
    }
}

// Event handlers
refreshBtn.addEventListener('click', refreshProviders);

settingsBtn.addEventListener('click', async () => {
    try {
        await invoke('open_settings');
    } catch (error) {
        console.error('Failed to open settings:', error);
    }
});

aboutBtn.addEventListener('click', async () => {
    try {
        await invoke('open_about');
    } catch (error) {
        console.error('Failed to open about:', error);
    }
});

cookiesBtn.addEventListener('click', async () => {
    try {
        await invoke('open_cookie_input');
    } catch (error) {
        console.error('Failed to open cookies:', error);
    }
});

quitBtn.addEventListener('click', async () => {
    try {
        await invoke('quit_app');
    } catch (error) {
        console.error('Failed to quit:', error);
    }
});

// Provider click handler (for potential future expansion)
providersContainer.addEventListener('click', (e) => {
    const provider = e.target.closest('.provider');
    if (provider) {
        const name = provider.dataset.name;
        console.log('Clicked provider:', name);
        // Could open provider details or website
    }
});

// Initial load
document.addEventListener('DOMContentLoaded', () => {
    refreshProviders();
});

// Auto-refresh every 60 seconds
setInterval(refreshProviders, 60000);

// Listen for Tauri events
window.__TAURI__.event.listen('providers-updated', (event) => {
    renderProviders(event.payload);
    lastUpdatedSpan.textContent = 'Updated just now';
});

// Handle window visibility
document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
        refreshProviders();
    }
});
