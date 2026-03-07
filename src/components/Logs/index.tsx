import { useEffect, useState, useRef, useCallback } from 'react';
import { motion } from 'framer-motion';
import {
  Trash2,
  RefreshCw,
  Download,
  Filter,
  Terminal,
  Search,
  X,
  Server,
  Monitor,
} from 'lucide-react';
import clsx from 'clsx';
import { logStore, LogEntry } from '../../lib/logger';
import { api, isTauri } from '../../lib/tauri';

type FilterLevel = 'all' | 'debug' | 'info' | 'warn' | 'error';
type LogTab = 'frontend' | 'backend';

const LEVEL_COLORS: Record<string, string> = {
  debug: 'text-gray-400',
  info: 'text-green-400',
  warn: 'text-yellow-400',
  error: 'text-red-400',
};

const LEVEL_BG: Record<string, string> = {
  debug: 'bg-gray-500/10',
  info: 'bg-green-500/10',
  warn: 'bg-yellow-500/10',
  error: 'bg-red-500/10',
};

const MODULE_COLORS: Record<string, string> = {
  App: 'text-purple-400',
  Service: 'text-blue-400',
  Config: 'text-emerald-400',
  AI: 'text-pink-400',
  Channel: 'text-orange-400',
  Setup: 'text-cyan-400',
  Dashboard: 'text-lime-400',
  Testing: 'text-fuchsia-400',
  API: 'text-amber-400',
};

const getBackendLineClass = (line: string) => {
  if (line.includes('error') || line.includes('Error') || line.includes('ERROR')) return 'text-red-400';
  if (line.includes('warn') || line.includes('Warn') || line.includes('WARN')) return 'text-yellow-400';
  if (line.includes('info') || line.includes('Info') || line.includes('INFO')) return 'text-green-400';
  return 'text-gray-400';
};

export function Logs() {
  const [activeTab, setActiveTab] = useState<LogTab>('frontend');

  // Frontend logs state
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<FilterLevel>('all');
  const [moduleFilter, setModuleFilter] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [autoScroll, setAutoScroll] = useState(true);
  const logsEndRef = useRef<HTMLDivElement>(null);

  // Backend logs state
  const [backendLogs, setBackendLogs] = useState<string[]>([]);
  const [backendSearch, setBackendSearch] = useState('');
  const [backendAutoRefresh, setBackendAutoRefresh] = useState(true);
  const [backendLoading, setBackendLoading] = useState(false);
  const backendEndRef = useRef<HTMLDivElement>(null);

  // Subscribe to frontend log updates
  useEffect(() => {
    const updateLogs = () => setLogs(logStore.getAll());
    updateLogs();
    return logStore.subscribe(updateLogs);
  }, []);

  // Auto scroll frontend
  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  // Fetch backend logs
  const fetchBackendLogs = useCallback(async () => {
    if (!isTauri()) return;
    setBackendLoading(true);
    try {
      const result = await api.getLogs(200);
      setBackendLogs(result);
    } catch {
      // silently ignore
    } finally {
      setBackendLoading(false);
    }
  }, []);

  // Auto refresh backend logs
  useEffect(() => {
    if (activeTab !== 'backend') return;
    fetchBackendLogs();
    if (!backendAutoRefresh) return;
    const interval = setInterval(fetchBackendLogs, 3000);
    return () => clearInterval(interval);
  }, [activeTab, backendAutoRefresh, fetchBackendLogs]);

  // Auto scroll backend
  useEffect(() => {
    if (activeTab === 'backend' && backendEndRef.current) {
      backendEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [backendLogs, activeTab]);

  // Filter frontend logs
  const filteredLogs = logs.filter(log => {
    if (filter !== 'all' && log.level !== filter) return false;
    if (moduleFilter !== 'all' && log.module !== moduleFilter) return false;
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      if (!log.message.toLowerCase().includes(q) && !log.module.toLowerCase().includes(q)) return false;
    }
    return true;
  });

  // Filter backend logs
  const filteredBackendLogs = backendLogs.filter(line =>
    !backendSearch.trim() || line.toLowerCase().includes(backendSearch.toLowerCase())
  );

  const modules = [...new Set(logs.map(log => log.module))];

  const handleClear = () => logStore.clear();

  const handleExport = () => {
    const content = filteredLogs.map(log => {
      const time = log.timestamp.toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
      const args = log.args.length > 0 ? ' ' + JSON.stringify(log.args) : '';
      return `[${time}] [${log.level.toUpperCase()}] [${log.module}] ${log.message}${args}`;
    }).join('\n');
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `openclaw-manager-logs-${new Date().toISOString().slice(0, 10)}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleExportBackend = () => {
    const content = filteredBackendLogs.join('\n');
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `openclaw-backend-logs-${new Date().toISOString().slice(0, 10)}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const formatTime = (date: Date) =>
    date.toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }) +
    '.' + String(date.getMilliseconds()).padStart(3, '0');

  const formatArgs = (args: unknown[]): string => {
    if (args.length === 0) return '';
    try {
      return args.map(arg => typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)).join(' ');
    } catch {
      return '[Cannot serialize]';
    }
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Tabs */}
      <div className="flex items-center gap-2 mb-4">
        <button
          onClick={() => setActiveTab('frontend')}
          className={clsx(
            'flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all',
            activeTab === 'frontend'
              ? 'bg-dark-600 text-white border border-dark-500'
              : 'text-gray-400 hover:text-white hover:bg-dark-700'
          )}
        >
          <Monitor size={15} />
          Frontend Logs
          {logs.filter(l => l.level === 'error').length > 0 && (
            <span className="bg-red-500/20 text-red-400 text-xs px-1.5 py-0.5 rounded-full">
              {logs.filter(l => l.level === 'error').length}
            </span>
          )}
        </button>
        <button
          onClick={() => setActiveTab('backend')}
          className={clsx(
            'flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all',
            activeTab === 'backend'
              ? 'bg-dark-600 text-white border border-dark-500'
              : 'text-gray-400 hover:text-white hover:bg-dark-700'
          )}
        >
          <Server size={15} />
          Backend Logs
        </button>
      </div>

      {activeTab === 'frontend' && (
        <>
          {/* Toolbar */}
          <div className="flex items-center gap-3 mb-4 flex-wrap">
            {/* Search */}
            <div className="relative">
              <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search logs..."
                className="bg-dark-700 border border-dark-500 rounded-lg pl-8 pr-8 py-1.5 text-sm text-gray-300 w-44 focus:outline-none focus:border-claw-500"
              />
              {searchQuery && (
                <button onClick={() => setSearchQuery('')} className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300">
                  <X size={13} />
                </button>
              )}
            </div>

            {/* Level filter */}
            <div className="flex items-center gap-2">
              <Filter size={14} className="text-gray-500" />
              <select
                value={filter}
                onChange={(e) => setFilter(e.target.value as FilterLevel)}
                className="bg-dark-700 border border-dark-500 rounded-lg px-3 py-1.5 text-sm text-gray-300"
              >
                <option value="all">All Levels</option>
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warn</option>
                <option value="error">Error</option>
              </select>
            </div>

            {/* Module filter */}
            <select
              value={moduleFilter}
              onChange={(e) => setModuleFilter(e.target.value)}
              className="bg-dark-700 border border-dark-500 rounded-lg px-3 py-1.5 text-sm text-gray-300"
            >
              <option value="all">All Modules</option>
              {modules.map(module => (
                <option key={module} value={module}>{module}</option>
              ))}
            </select>

            <div className="flex-1" />

            {/* Statistics */}
            <div className="flex items-center gap-3 text-xs text-gray-500">
              <span>{filteredLogs.length} / {logs.length} entries</span>
              <span className="text-red-400">{logs.filter(l => l.level === 'error').length} errors</span>
              <span className="text-yellow-400">{logs.filter(l => l.level === 'warn').length} warnings</span>
            </div>

            {/* Action buttons */}
            <div className="flex items-center gap-2">
              <label className="flex items-center gap-1 text-xs text-gray-400">
                <input
                  type="checkbox"
                  checked={autoScroll}
                  onChange={(e) => setAutoScroll(e.target.checked)}
                  className="w-3 h-3 rounded"
                />
                Auto scroll
              </label>
              <button onClick={handleExport} className="icon-button text-gray-400 hover:text-white" title="Export logs">
                <Download size={16} />
              </button>
              <button onClick={() => setLogs(logStore.getAll())} className="icon-button text-gray-400 hover:text-white" title="Refresh">
                <RefreshCw size={16} />
              </button>
              <button onClick={handleClear} className="icon-button text-gray-400 hover:text-red-400" title="Clear logs">
                <Trash2 size={16} />
              </button>
            </div>
          </div>

          {/* Log list */}
          <div className="flex-1 bg-dark-800 rounded-xl border border-dark-600 overflow-hidden flex flex-col">
            <div className="flex items-center gap-2 px-4 py-2 bg-dark-700 border-b border-dark-600">
              <Terminal size={14} className="text-gray-500" />
              <span className="text-xs text-gray-400 font-medium">Application Logs</span>
              {searchQuery && (
                <span className="text-xs text-claw-400 ml-2">— searching: "{searchQuery}"</span>
              )}
            </div>
            <div className="flex-1 overflow-y-auto p-2 font-mono text-xs">
              {filteredLogs.length === 0 ? (
                <div className="h-full flex items-center justify-center text-gray-500">
                  <div className="text-center">
                    <Terminal size={32} className="mx-auto mb-2 opacity-50" />
                    <p>{searchQuery ? 'No results matching your search' : 'No logs available'}</p>
                  </div>
                </div>
              ) : (
                <>
                  {filteredLogs.map((log) => (
                    <motion.div
                      key={log.id}
                      initial={{ opacity: 0, x: -10 }}
                      animate={{ opacity: 1, x: 0 }}
                      className={clsx('py-1.5 px-2 rounded mb-1', LEVEL_BG[log.level])}
                    >
                      <div className="flex items-start gap-2">
                        <span className="text-gray-600 flex-shrink-0">{formatTime(log.timestamp)}</span>
                        <span className={clsx('px-1.5 py-0.5 rounded text-[10px] uppercase flex-shrink-0', LEVEL_COLORS[log.level])}>
                          {log.level}
                        </span>
                        <span className={clsx('flex-shrink-0', MODULE_COLORS[log.module] || 'text-gray-400')}>
                          [{log.module}]
                        </span>
                        <span className="text-gray-300 break-all">{log.message}</span>
                      </div>
                      {log.args.length > 0 && (
                        <div className="mt-1 ml-20 text-gray-500 break-all whitespace-pre-wrap">
                          {formatArgs(log.args)}
                        </div>
                      )}
                    </motion.div>
                  ))}
                  <div ref={logsEndRef} />
                </>
              )}
            </div>
          </div>
        </>
      )}

      {activeTab === 'backend' && (
        <>
          {/* Backend toolbar */}
          <div className="flex items-center gap-3 mb-4 flex-wrap">
            <div className="relative">
              <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
              <input
                type="text"
                value={backendSearch}
                onChange={(e) => setBackendSearch(e.target.value)}
                placeholder="Search backend logs..."
                className="bg-dark-700 border border-dark-500 rounded-lg pl-8 pr-8 py-1.5 text-sm text-gray-300 w-52 focus:outline-none focus:border-claw-500"
              />
              {backendSearch && (
                <button onClick={() => setBackendSearch('')} className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300">
                  <X size={13} />
                </button>
              )}
            </div>

            <div className="flex-1" />

            <span className="text-xs text-gray-500">{filteredBackendLogs.length} / {backendLogs.length} lines</span>

            <label className="flex items-center gap-1 text-xs text-gray-400">
              <input
                type="checkbox"
                checked={backendAutoRefresh}
                onChange={(e) => setBackendAutoRefresh(e.target.checked)}
                className="w-3 h-3 rounded"
              />
              Auto refresh
            </label>
            <button onClick={handleExportBackend} className="icon-button text-gray-400 hover:text-white" title="Export">
              <Download size={16} />
            </button>
            <button onClick={fetchBackendLogs} className={clsx('icon-button text-gray-400 hover:text-white', backendLoading && 'animate-spin')} title="Refresh">
              <RefreshCw size={16} />
            </button>
          </div>

          {/* Backend log viewer */}
          <div className="flex-1 bg-dark-800 rounded-xl border border-dark-600 overflow-hidden flex flex-col">
            <div className="flex items-center gap-2 px-4 py-2 bg-dark-700 border-b border-dark-600">
              <Server size={14} className="text-gray-500" />
              <span className="text-xs text-gray-400 font-medium">/tmp/openclaw-gateway.log</span>
              {backendSearch && (
                <span className="text-xs text-claw-400 ml-2">— searching: "{backendSearch}"</span>
              )}
            </div>
            <div className="flex-1 overflow-y-auto p-4 font-mono text-xs leading-relaxed">
              {filteredBackendLogs.length === 0 ? (
                <div className="h-full flex items-center justify-center text-gray-500">
                  <div className="text-center">
                    <Server size={32} className="mx-auto mb-2 opacity-50" />
                    <p>{backendSearch ? 'No results matching your search' : 'No backend logs available'}</p>
                  </div>
                </div>
              ) : (
                <>
                  {filteredBackendLogs.map((line, index) => (
                    <div key={index} className={clsx('py-0.5', getBackendLineClass(line))}>
                      <span className="text-gray-600 mr-3 select-none">{String(index + 1).padStart(4, ' ')}</span>
                      {line}
                    </div>
                  ))}
                  <div ref={backendEndRef} />
                </>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
