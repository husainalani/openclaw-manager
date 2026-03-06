import React, { useCallback, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Sidebar } from './components/Layout/Sidebar';
import { Header } from './components/Layout/Header';
import { Dashboard } from './components/Dashboard';
import { AIConfig } from './components/AIConfig';
import { Channels } from './components/Channels';
import { MCP } from './components/MCP';
import { Skills } from './components/Skills';
import { Settings } from './components/Settings';
import { Logs } from './components/Logs';
import { Agents } from './components/Agents';
import { appLogger } from './lib/logger';
import { isTauri } from './lib/tauri';
import { Download, X, Loader2, CheckCircle, AlertCircle } from 'lucide-react';
import { useAppStore } from './stores/appStore';
import { useEnvironment } from './hooks/useEnvironment';
import { useOpenClawUpdate } from './hooks/useOpenClawUpdate';
import { useManagerUpdate } from './hooks/useManagerUpdate';
import { useSecurityCheck } from './hooks/useSecurityCheck';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ServiceStatus } from './lib/tauri';

export type { EnvironmentStatus } from './lib/tauri';
export type PageType = 'dashboard' | 'mcp' | 'skills' | 'ai' | 'channels' | 'agents' | 'logs' | 'settings';

class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { hasError: boolean, error: Error | null }> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    appLogger.error('ErrorBoundary caught error', { error, errorInfo });
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-8 text-center">
          <AlertCircle size={48} className="mx-auto text-red-400 mb-4" />
          <h2 className="text-xl font-bold text-white mb-2">Something went wrong</h2>
          <p className="text-red-200 mb-4">{this.state.error?.message}</p>
          <button
            onClick={() => this.setState({ hasError: false })}
            className="px-4 py-2 bg-dark-700 hover:bg-dark-600 rounded-lg text-white text-sm"
          >
            Try again
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

function App() {
  const [currentPage, setCurrentPage] = useState<PageType>('dashboard');
  const [serviceStatus, setServiceStatus] = useState<ServiceStatus | null>(null);
  const { setServiceStatus: storeSetServiceStatus } = useAppStore();

  const { isReady, envStatus, checkEnvironment } = useEnvironment();

  const { updateInfo, showUpdateBanner, updating, updateResult, checkUpdate, handleUpdate, dismissUpdateBanner } =
    useOpenClawUpdate(checkEnvironment);

  const {
    managerUpdateAvailable,
    managerUpdateVersion,
    showManagerUpdateBanner,
    managerUpdating,
    managerUpdateProgress,
    managerUpdateResult,
    checkManagerUpdate,
    handleManagerUpdate,
    dismissManagerUpdateBanner,
  } = useManagerUpdate();

  const { secureVersionInfo, showSecurityBanner, setShowSecurityBanner, checkSecurity } = useSecurityCheck();

  useEffect(() => {
    appLogger.info('🦞 App component mounted');
    checkEnvironment();
  }, [checkEnvironment]);

  // Delay update/security checks after startup to avoid blocking startup
  useEffect(() => {
    if (!isTauri()) return;
    const t1 = setTimeout(() => { checkSecurity(); }, 1000);
    const t2 = setTimeout(() => { checkUpdate(); }, 2000);
    const t3 = setTimeout(() => { checkManagerUpdate(); }, 6000);
    return () => { clearTimeout(t1); clearTimeout(t2); clearTimeout(t3); };
  }, [checkSecurity, checkUpdate, checkManagerUpdate]);

  // Periodically poll service status
  useEffect(() => {
    if (!isTauri()) return;

    const fetchServiceStatus = async () => {
      try {
        const status = await invoke<ServiceStatus>('get_service_status');
        setServiceStatus(status);
        storeSetServiceStatus(status);
      } catch {
        // Silently handle polling errors
      }
    };
    fetchServiceStatus();
    const interval = setInterval(fetchServiceStatus, 3000);
    return () => clearInterval(interval);
  }, [storeSetServiceStatus]);

  const handleSetupComplete = useCallback(() => {
    appLogger.info('Setup wizard completed');
    checkEnvironment();
  }, [checkEnvironment]);

  const handleNavigate = (page: PageType) => {
    appLogger.action('Page navigation', { from: currentPage, to: page });
    setCurrentPage(page);
  };

  const renderPage = () => {
    const pageVariants = {
      initial: { opacity: 0, x: 20 },
      animate: { opacity: 1, x: 0 },
      exit: { opacity: 0, x: -20 },
    };

    const pages: Record<PageType, JSX.Element> = {
      dashboard: <Dashboard envStatus={envStatus} onSetupComplete={handleSetupComplete} />,
      mcp: <MCP />,
      skills: <Skills />,
      ai: <AIConfig />,
      channels: <Channels />,
      agents: <Agents />,
      logs: <Logs />,
      settings: <Settings onEnvironmentChange={checkEnvironment} />,
    };

    return (
      <AnimatePresence mode="wait">
        <motion.div
          key={currentPage}
          variants={pageVariants}
          initial="initial"
          animate="animate"
          exit="exit"
          transition={{ duration: 0.2 }}
          className="h-full"
        >
          {pages[currentPage]}
        </motion.div>
      </AnimatePresence>
    );
  };

  if (isReady === null) {
    return (
      <div className="flex h-screen bg-dark-900 items-center justify-center">
        <div className="fixed inset-0 bg-gradient-radial pointer-events-none" />
        <div className="relative z-10 text-center">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-xl bg-gradient-to-br from-brand-500 to-purple-600 mb-4 animate-pulse">
            <span className="text-3xl">🦞</span>
          </div>
          <p className="text-dark-400">Starting...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-dark-900 overflow-hidden">
      <div className="fixed inset-0 bg-gradient-radial pointer-events-none" />

      {/* Security Banner */}
      <AnimatePresence>
        {showSecurityBanner && secureVersionInfo && !secureVersionInfo.is_secure && (
          <motion.div
            initial={{ opacity: 0, y: -50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -50 }}
            className="fixed top-0 left-0 right-0 z-[60] bg-gradient-to-r from-red-600 to-orange-600 shadow-lg"
          >
            <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <AlertCircle size={20} className="text-white" />
                <div>
                  <p className="text-sm font-bold text-white">
                    Security Warning: Your OpenClaw version ({secureVersionInfo.current_version}) is insecure.
                  </p>
                  <p className="text-xs text-white/90">
                    A version &ge; 2026.1.29 is required. Please update immediately.
                  </p>
                </div>
              </div>
              <button
                onClick={() => setShowSecurityBanner(false)}
                className="p-1.5 hover:bg-white/20 rounded-lg transition-colors text-white/90 hover:text-white"
              >
                <X size={16} />
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* OpenClaw Update Banner */}
      <AnimatePresence>
        {showUpdateBanner && updateInfo?.update_available && (
          <motion.div
            initial={{ opacity: 0, y: -50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -50 }}
            className="fixed top-0 left-0 right-0 z-50 bg-gradient-to-r from-claw-600 to-purple-600 shadow-lg"
          >
            <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-3">
                {updateResult?.success ? (
                  <CheckCircle size={20} className="text-green-300" />
                ) : updateResult && !updateResult.success ? (
                  <AlertCircle size={20} className="text-red-300" />
                ) : (
                  <Download size={20} className="text-white" />
                )}
                <div>
                  {updateResult ? (
                    <p className={`text-sm font-medium ${updateResult.success ? 'text-green-100' : 'text-red-100'}`}>
                      {updateResult.message}
                    </p>
                  ) : (
                    <>
                      <p className="text-sm font-medium text-white">
                        New version available: OpenClaw {updateInfo.latest_version}
                      </p>
                      <p className="text-xs text-white/70">
                        Current version: {updateInfo.current_version}
                      </p>
                    </>
                  )}
                </div>
              </div>
              <div className="flex items-center gap-2">
                {!updateResult && (
                  <button
                    onClick={handleUpdate}
                    disabled={updating}
                    className="px-4 py-1.5 bg-white/20 hover:bg-white/30 text-white text-sm font-medium rounded-lg transition-colors flex items-center gap-2 disabled:opacity-50"
                  >
                    {updating ? (
                      <><Loader2 size={14} className="animate-spin" />Updating...</>
                    ) : (
                      <><Download size={14} />Update Now</>
                    )}
                  </button>
                )}
                <button onClick={dismissUpdateBanner} className="p-1.5 hover:bg-white/20 rounded-lg transition-colors text-white/70 hover:text-white">
                  <X size={16} />
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Manager Update Banner */}
      <AnimatePresence>
        {showManagerUpdateBanner && managerUpdateAvailable && (
          <motion.div
            initial={{ opacity: 0, y: -50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -50 }}
            className="fixed top-0 left-0 right-0 z-[45] bg-gradient-to-r from-emerald-600 to-teal-600 shadow-lg"
          >
            <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-3 w-1/2">
                {managerUpdateResult?.success ? (
                  <CheckCircle size={20} className="text-green-300 shrink-0" />
                ) : managerUpdateResult && !managerUpdateResult.success ? (
                  <AlertCircle size={20} className="text-red-300 shrink-0" />
                ) : (
                  <Download size={20} className="text-white shrink-0" />
                )}
                <div className="flex-1">
                  {managerUpdateResult ? (
                    <p className={`text-sm font-medium ${managerUpdateResult.success ? 'text-green-100' : 'text-red-100'}`}>
                      {managerUpdateResult.message}
                    </p>
                  ) : (
                    <>
                      <div className="flex justify-between items-center pr-4">
                        <p className="text-sm font-medium text-white">
                          New version available: Manager v{managerUpdateVersion}
                        </p>
                        {managerUpdating && (
                          <span className="text-xs text-white/80">{managerUpdateProgress}%</span>
                        )}
                      </div>
                      {managerUpdating && (
                        <div className="w-full bg-black/20 rounded-full h-1 mt-1.5 mr-4 max-w-[200px]">
                          <div
                            className="bg-white h-1 rounded-full transition-all duration-300"
                            style={{ width: `${managerUpdateProgress}%` }}
                          />
                        </div>
                      )}
                    </>
                  )}
                </div>
              </div>
              <div className="flex items-center gap-2">
                {!managerUpdateResult && (
                  <button
                    onClick={handleManagerUpdate}
                    disabled={managerUpdating}
                    className="px-4 py-1.5 bg-white/20 hover:bg-white/30 text-white text-sm font-medium rounded-lg transition-colors flex items-center gap-2 disabled:opacity-50"
                  >
                    {managerUpdating ? (
                      <><Loader2 size={14} className="animate-spin" />Updating...</>
                    ) : (
                      <><Download size={14} />Update Now</>
                    )}
                  </button>
                )}
                <button onClick={dismissManagerUpdateBanner} className="p-1.5 hover:bg-white/20 rounded-lg transition-colors text-white/70 hover:text-white">
                  <X size={16} />
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <Sidebar currentPage={currentPage} onNavigate={handleNavigate} serviceStatus={serviceStatus} />

      <div className="flex-1 flex flex-col overflow-hidden">
        <Header currentPage={currentPage} />
        <main className="flex-1 overflow-hidden p-6">
          <ErrorBoundary>
            {renderPage()}
          </ErrorBoundary>
        </main>
      </div>
    </div>
  );
}

export default App;
