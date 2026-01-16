// CodexBar Tauri Frontend - Tab-based UI

const { invoke } = window.__TAURI__.core;

// Provider icons mapping
const PROVIDER_ICONS = {
    codex: '⚡',
    claude: '✦',
    cursor: '⌘',
    gemini: '◇',
    copilot: '⊕',
    antigravity: '✧',
    factory: '⚙',
    zed: 'Z',
    kiro: 'K',
    vertexai: '△',
    augment: 'A',
    minimax: 'M',
    opencode: 'O'
};

// State
let providers = [];
let selectedProvider = null;
let isRefreshing = false;

// DOM Elements
const tabBar = document.getElementById('tabBar');
const providerDetail = document.getElementById('providerDetail');
const detailName = document.getElementById('detailName');
const detailUpdated = document.getElementById('detailUpdated');
const detailPlan = document.getElementById('detailPlan');

const sessionSection = document.getElementById('sessionSection');
const sessionProgress = document.getElementById('sessionProgress');
const sessionPercent = document.getElementById('sessionPercent');
const sessionReset = document.getElementById('sessionReset');

const weeklySection = document.getElementById('weeklySection');
const weeklyProgress = document.getElementById('weeklyProgress');
const weeklyPercent = document.getElementById('weeklyPercent');
const weeklyReset = document.getElementById('weeklyReset');
const paceInfo = document.getElementById('paceInfo');

const modelSection = document.getElementById('modelSection');
const modelTitle = document.getElementById('modelTitle');
const modelPercent = document.getElementById('modelPercent');

const extraSection = document.getElementById('extraSection');
const extraInfo = document.getElementById('extraInfo');
const extraPercent = document.getElementById('extraPercent');

const costSection = document.getElementById('costSection');
const costToday = document.getElementById('costToday');
const costMonth = document.getElementById('costMonth');

const errorSection = document.getElementById('errorSection');
const errorMessage = document.getElementById('errorMessage');

const settingsBtn = document.getElementById('settingsBtn');
const aboutBtn = document.getElementById('aboutBtn');
const quitBtn = document.getElementById('quitBtn');
const dashboardBtn = document.getElementById('dashboardBtn');
const statusPageBtn = document.getElementById('statusPageBtn');
const addAccountBtn = document.getElementById('addAccountBtn');

// Color mapping based on percentage
function getColorClass(percent) {
    if (percent === null || percent === undefined) return 'gray';
    if (percent <= 25) return 'green';
    if (percent <= 50) return 'yellow';
    if (percent <= 75) return 'orange';
    return 'red';
}

// Format reset time
function formatResetTime(resetAt) {
    if (!resetAt) return '';

    const reset = new Date(resetAt);
    const now = new Date();
    const diff = reset - now;

    if (diff <= 0) return 'Resetting...';

    const hours = Math.floor(diff / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

    if (hours >= 24) {
        const days = Math.floor(hours / 24);
        const remainingHours = hours % 24;
        return `Resets in ${days}d ${remainingHours}h`;
    }

    return `Resets in ${hours}h ${minutes}m`;
}

// Format tokens
function formatTokens(tokens) {
    if (!tokens) return '0';
    if (tokens >= 1000000000) return `${(tokens / 1000000000).toFixed(1)}B`;
    if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(0)}M`;
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(0)}K`;
    return tokens.toString();
}

// Render tab bar - show providers that have data
function renderTabs() {
    // Only show providers that successfully returned data (no error)
    const successfulProviders = providers.filter(p => p.percent !== null && !p.error);

    // Fall back to all providers if none successful yet
    const tabProviders = successfulProviders.length > 0 ? successfulProviders : providers.slice(0, 2);

    if (tabProviders.length === 0) {
        tabBar.innerHTML = '<div style="padding: 10px; color: #666;">Loading...</div>';
        return;
    }

    tabBar.innerHTML = tabProviders
        .map(p => {
            const isActive = selectedProvider && selectedProvider.name === p.name;
            const icon = PROVIDER_ICONS[p.name.toLowerCase()] || p.name.charAt(0).toUpperCase();

            return `
                <button class="tab ${isActive ? 'active' : ''}"
                        data-provider="${p.name}">
                    <span class="tab-icon">${icon}</span>
                    <span class="tab-label">${p.name}</span>
                </button>
            `;
        }).join('');

    // Add click handlers
    tabBar.querySelectorAll('.tab').forEach(tab => {
        tab.addEventListener('click', () => {
            const providerName = tab.dataset.provider;
            const provider = providers.find(p => p.name === providerName);
            if (provider) {
                selectProvider(provider);
            }
        });
    });
}

// Select and display a provider
function selectProvider(provider) {
    selectedProvider = provider;
    renderTabs();
    renderProviderDetail(provider);
}

// Render provider detail view
function renderProviderDetail(provider) {
    if (!provider) {
        providerDetail.innerHTML = '<div class="loading">Select a provider</div>';
        return;
    }

    // Header
    detailName.textContent = provider.displayName || provider.name;
    detailUpdated.textContent = 'Updated just now';
    detailPlan.textContent = provider.plan || '';

    // Handle error state
    if (provider.error) {
        sessionSection.classList.add('hidden');
        weeklySection.classList.add('hidden');
        modelSection.classList.add('hidden');
        extraSection.classList.add('hidden');
        costSection.classList.add('hidden');

        errorSection.classList.add('visible');
        errorMessage.textContent = provider.error;
        return;
    }

    errorSection.classList.remove('visible');

    // Session usage
    if (provider.session !== undefined && provider.session !== null) {
        sessionSection.classList.remove('hidden');
        const sessionColor = getColorClass(provider.session);
        sessionProgress.style.width = `${provider.session}%`;
        sessionProgress.className = `progress-fill ${sessionColor}`;
        sessionPercent.textContent = `${provider.session}% used`;
        sessionReset.textContent = formatResetTime(provider.sessionReset);
    } else if (provider.percent !== undefined && provider.percent !== null) {
        // Fallback to single percent
        sessionSection.classList.remove('hidden');
        const sessionColor = getColorClass(provider.percent);
        sessionProgress.style.width = `${provider.percent}%`;
        sessionProgress.className = `progress-fill ${sessionColor}`;
        sessionPercent.textContent = `${provider.percent}% used`;
        sessionReset.textContent = formatResetTime(provider.resetAt);
    } else {
        sessionSection.classList.add('hidden');
    }

    // Weekly usage
    if (provider.weekly !== undefined && provider.weekly !== null) {
        weeklySection.classList.remove('hidden');
        const weeklyColor = getColorClass(provider.weekly);
        weeklyProgress.style.width = `${provider.weekly}%`;
        weeklyProgress.className = `progress-fill ${weeklyColor}`;
        weeklyPercent.textContent = `${provider.weekly}% used`;
        weeklyReset.textContent = formatResetTime(provider.weeklyReset);

        // Pace info
        if (provider.pace) {
            paceInfo.textContent = provider.pace;
            paceInfo.classList.remove('hidden');
        } else {
            paceInfo.classList.add('hidden');
        }
    } else {
        weeklySection.classList.add('hidden');
    }

    // Model-specific usage (Sonnet/Opus)
    if (provider.model !== undefined && provider.model !== null) {
        modelSection.classList.remove('hidden');
        modelTitle.textContent = provider.modelName || 'Model';
        modelPercent.textContent = `${provider.model}% used`;
    } else {
        modelSection.classList.add('hidden');
    }

    // Extra usage (billing)
    if (provider.extraUsage !== undefined) {
        extraSection.classList.remove('hidden');
        extraInfo.textContent = `This month: $${provider.extraUsed?.toFixed(2) || '0.00'} / $${provider.extraLimit?.toFixed(2) || '0.00'}`;
        extraPercent.textContent = `${provider.extraUsage}% used`;
    } else {
        extraSection.classList.add('hidden');
    }

    // Cost section
    if (provider.cost) {
        costSection.classList.remove('hidden');
        costToday.textContent = `Today: $${provider.cost.today?.toFixed(2) || '0.00'} · ${formatTokens(provider.cost.todayTokens)} tokens`;
        costMonth.textContent = `Last 30 days: $${provider.cost.month?.toFixed(2) || '0.00'} · ${formatTokens(provider.cost.monthTokens)} tokens`;
    } else {
        costSection.classList.add('hidden');
    }
}

// Fetch and update provider data
async function refreshProviders() {
    if (isRefreshing) return;

    isRefreshing = true;

    try {
        const result = await invoke('get_providers');
        providers = result;

        renderTabs();

        // Select first provider if none selected, or refresh current selection
        if (!selectedProvider && providers.length > 0) {
            // Prefer first provider with data
            const withData = providers.find(p => p.percent !== null && !p.error);
            selectProvider(withData || providers[0]);
        } else if (selectedProvider) {
            // Refresh current selection
            const updated = providers.find(p => p.name === selectedProvider.name);
            if (updated) {
                selectProvider(updated);
            }
        }
    } catch (error) {
        console.error('Failed to fetch providers:', error);
    } finally {
        isRefreshing = false;
    }
}

// Event handlers
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

quitBtn.addEventListener('click', async () => {
    try {
        await invoke('quit_app');
    } catch (error) {
        console.error('Failed to quit:', error);
    }
});

dashboardBtn.addEventListener('click', async () => {
    if (selectedProvider && selectedProvider.dashboardUrl) {
        try {
            await invoke('open_url', { url: selectedProvider.dashboardUrl });
        } catch (error) {
            console.error('Failed to open dashboard:', error);
        }
    }
});

statusPageBtn.addEventListener('click', async () => {
    if (selectedProvider && selectedProvider.statusPageUrl) {
        try {
            await invoke('open_url', { url: selectedProvider.statusPageUrl });
        } catch (error) {
            console.error('Failed to open status page:', error);
        }
    }
});

addAccountBtn.addEventListener('click', async () => {
    try {
        await invoke('open_cookie_input');
    } catch (error) {
        console.error('Failed to open add account:', error);
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
    providers = event.payload;
    renderTabs();
    if (selectedProvider) {
        const updated = providers.find(p => p.name === selectedProvider.name);
        if (updated) {
            renderProviderDetail(updated);
        }
    }
});

// Handle window visibility
document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
        refreshProviders();
    }
});
