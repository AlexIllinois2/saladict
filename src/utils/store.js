import { Store } from '@tauri-apps/plugin-store';
import { appConfigDir, join } from '@tauri-apps/api/path';
import { watch } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';

// v2: Store uses a private constructor; you must use `Store.load(path)`.
// Initialized lazily in initStore(); null until then.
export let store = null;

export async function initStore() {
    const appConfigDirPath = await appConfigDir();
    const appConfigPath = await join(appConfigDirPath, 'config.json');
    store = await Store.load(appConfigPath);
    // Don't let a missing `watch` permission (or any error) block app startup.
    try {
        await watch(appConfigPath, async () => {
            await store.reload();
            await invoke('reload_store');
        });
    } catch (e) {
        console.warn('Failed to watch config file:', e);
    }
}
