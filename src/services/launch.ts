import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const PATHS_AVAILABLE_EVENT = 'database-paths-available';

export function takePendingDatabasePaths(): Promise<string[]> {
    return invoke('take_pending_database_paths');
}

export function onDatabasePathsAvailable(callback: () => void): ReturnType<typeof listen> {
    return listen(PATHS_AVAILABLE_EVENT, callback);
}
