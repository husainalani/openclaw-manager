import { useState, useCallback } from 'react';
import { isTauri, type UpdateResult } from '../lib/tauri';
import { appLogger } from '../lib/logger';

export function useManagerUpdate() {
  const [managerUpdateAvailable, setManagerUpdateAvailable] = useState(false);
  const [managerUpdateVersion, setManagerUpdateVersion] = useState<string | null>(null);
  const [showManagerUpdateBanner, setShowManagerUpdateBanner] = useState(false);
  const [managerUpdating, setManagerUpdating] = useState(false);
  const [managerUpdateProgress, setManagerUpdateProgress] = useState(0);
  const [managerUpdateResult, setManagerUpdateResult] = useState<UpdateResult | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [managerUpdateObj, setManagerUpdateObj] = useState<any>(null);

  const checkManagerUpdate = useCallback(async () => {
    if (!isTauri()) return;
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        setManagerUpdateAvailable(true);
        setManagerUpdateVersion(update.version);
        setManagerUpdateObj(update);
        setShowManagerUpdateBanner(true);
      }
    } catch (e) {
      appLogger.error('Manager update check failed', e);
    }
  }, []);

  const handleManagerUpdate = async () => {
    if (!managerUpdateObj) return;
    setManagerUpdating(true);
    setManagerUpdateProgress(0);
    setManagerUpdateResult(null);
    try {
      let downloaded = 0;
      let contentLength = 1;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await managerUpdateObj.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength || 1;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            setManagerUpdateProgress(Math.min(100, Math.round((downloaded / contentLength) * 100)));
            break;
          case 'Finished':
            setManagerUpdateProgress(100);
            break;
        }
      });
      setManagerUpdateResult({ success: true, message: 'Update installed successfully! Restarting...' });
      setTimeout(async () => {
        try {
          const { relaunch } = await import('@tauri-apps/plugin-process');
          await relaunch();
        } catch (err) {
          appLogger.error('Relaunch failed', err);
        }
      }, 2000);
    } catch (e: unknown) {
      const err = e as { message?: string };
      appLogger.error('Manager update download failed', e);
      setManagerUpdateResult({ success: false, message: 'Update failed', error: err?.message || String(e) });
      setManagerUpdating(false);
    }
  };

  const dismissManagerUpdateBanner = () => {
    setShowManagerUpdateBanner(false);
    setManagerUpdateResult(null);
  };

  return {
    managerUpdateAvailable,
    managerUpdateVersion,
    showManagerUpdateBanner,
    managerUpdating,
    managerUpdateProgress,
    managerUpdateResult,
    checkManagerUpdate,
    handleManagerUpdate,
    dismissManagerUpdateBanner,
  };
}
