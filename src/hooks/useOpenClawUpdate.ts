import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { isTauri, type UpdateInfo, type UpdateResult } from '../lib/tauri';
import { appLogger } from '../lib/logger';

export function useOpenClawUpdate(onUpdateSuccess: () => Promise<void>) {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateBanner, setShowUpdateBanner] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [updateResult, setUpdateResult] = useState<UpdateResult | null>(null);

  const checkUpdate = useCallback(async () => {
    if (!isTauri()) return;

    appLogger.info('Checking for OpenClaw updates...');
    try {
      const info = await invoke<UpdateInfo>('check_openclaw_update');
      appLogger.info('Update check result', info);
      setUpdateInfo(info);
      if (info.update_available) {
        setShowUpdateBanner(true);
      }
    } catch (e) {
      appLogger.error('Update check failed', e);
    }
  }, []);

  const handleUpdate = async () => {
    setUpdating(true);
    setUpdateResult(null);
    try {
      const result = await invoke<UpdateResult>('update_openclaw');
      setUpdateResult(result);
      if (result.success) {
        await onUpdateSuccess();
        setTimeout(() => {
          setShowUpdateBanner(false);
          setUpdateResult(null);
        }, 3000);
      }
    } catch (e) {
      setUpdateResult({
        success: false,
        message: 'Error occurred during update',
        error: String(e),
      });
    } finally {
      setUpdating(false);
    }
  };

  const dismissUpdateBanner = () => {
    setShowUpdateBanner(false);
    setUpdateResult(null);
  };

  return { updateInfo, showUpdateBanner, updating, updateResult, checkUpdate, handleUpdate, dismissUpdateBanner };
}
