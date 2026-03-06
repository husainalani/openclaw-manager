import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { isTauri, type EnvironmentStatus } from '../lib/tauri';
import { appLogger } from '../lib/logger';

export function useEnvironment() {
  const [isReady, setIsReady] = useState<boolean | null>(null);
  const [envStatus, setEnvStatus] = useState<EnvironmentStatus | null>(null);

  const checkEnvironment = useCallback(async () => {
    if (!isTauri()) {
      appLogger.warn('Not in Tauri environment, skipping environment check');
      setIsReady(true);
      return;
    }

    appLogger.info('Starting system environment check...');
    try {
      const status = await invoke<EnvironmentStatus>('check_environment');
      appLogger.info('Environment check completed', status);
      setEnvStatus(status);
      setIsReady(true);
    } catch (e) {
      appLogger.error('Environment check failed', e);
      setIsReady(true);
    }
  }, []);

  return { isReady, envStatus, checkEnvironment };
}
