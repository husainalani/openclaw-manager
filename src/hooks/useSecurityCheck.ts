import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { isTauri, type SecureVersionInfo } from '../lib/tauri';
import { appLogger } from '../lib/logger';

export function useSecurityCheck() {
  const [secureVersionInfo, setSecureVersionInfo] = useState<SecureVersionInfo | null>(null);
  const [showSecurityBanner, setShowSecurityBanner] = useState(false);

  const checkSecurity = useCallback(async () => {
    if (!isTauri()) return;

    appLogger.info('Checking OpenClaw version security...');
    try {
      const info = await invoke<SecureVersionInfo>('check_secure_version');
      appLogger.info('Security check result', info);
      setSecureVersionInfo(info);
      if (!info.is_secure) {
        setShowSecurityBanner(true);
      }
    } catch (e) {
      appLogger.error('Security check failed', e);
    }
  }, []);

  return { secureVersionInfo, showSecurityBanner, setShowSecurityBanner, checkSecurity };
}
