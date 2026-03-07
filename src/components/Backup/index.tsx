import { useState } from 'react';
import { motion } from 'framer-motion';
import { api, isTauri } from '../../lib/tauri';
import {
  Download,
  Upload,
  AlertCircle,
  CheckCircle,
  Loader2,
  ShieldCheck,
  FileJson,
  RotateCcw,
} from 'lucide-react';

export function Backup() {
  const [exportLoading, setExportLoading] = useState(false);
  const [importLoading, setImportLoading] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [jsonPreview, setJsonPreview] = useState<string | null>(null);
  const [showRestoreConfirm, setShowRestoreConfirm] = useState(false);
  const [pendingRestore, setPendingRestore] = useState<unknown>(null);

  const handleExport = async () => {
    if (!isTauri()) return;
    setExportLoading(true);
    setMessage(null);
    try {
      const config = await api.getConfig();
      const json = JSON.stringify(config, null, 2);
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `openclaw-backup-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      setMessage({ type: 'success', text: 'Configuration exported successfully.' });
    } catch (e) {
      setMessage({ type: 'error', text: 'Export failed: ' + String(e) });
    } finally {
      setExportLoading(false);
    }
  };

  const handleImportFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      try {
        const text = ev.target?.result as string;
        const parsed = JSON.parse(text);
        setJsonPreview(JSON.stringify(parsed, null, 2));
        setPendingRestore(parsed);
        setShowRestoreConfirm(true);
        setMessage(null);
      } catch {
        setMessage({ type: 'error', text: 'Invalid JSON file. Please select a valid backup file.' });
      }
    };
    reader.readAsText(file);
    e.target.value = '';
  };

  const handleConfirmRestore = async () => {
    if (!isTauri() || !pendingRestore) return;
    setImportLoading(true);
    setShowRestoreConfirm(false);
    setMessage(null);
    try {
      await api.saveConfig(pendingRestore);
      setMessage({ type: 'success', text: 'Configuration restored successfully. Restart the service for changes to take effect.' });
      setJsonPreview(null);
      setPendingRestore(null);
    } catch (e) {
      setMessage({ type: 'error', text: 'Restore failed: ' + String(e) });
    } finally {
      setImportLoading(false);
    }
  };

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2">
      <div className="max-w-3xl space-y-6">
        <div>
          <h2 className="text-2xl font-bold text-white mb-1">Backup & Restore</h2>
          <p className="text-gray-400 text-sm">Export or import your OpenClaw configuration file</p>
        </div>

        {/* Status message */}
        {message && (
          <motion.div
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            className={`flex items-start gap-3 p-4 rounded-xl border ${
              message.type === 'success'
                ? 'bg-green-500/10 border-green-500/30'
                : 'bg-red-500/10 border-red-500/30'
            }`}
          >
            {message.type === 'success'
              ? <CheckCircle size={18} className="text-green-400 mt-0.5 flex-shrink-0" />
              : <AlertCircle size={18} className="text-red-400 mt-0.5 flex-shrink-0" />}
            <p className={`text-sm ${message.type === 'success' ? 'text-green-300' : 'text-red-300'}`}>
              {message.text}
            </p>
          </motion.div>
        )}

        {/* Export card */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center">
              <Download size={20} className="text-cyan-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Export Configuration</h3>
              <p className="text-xs text-gray-500">Download a copy of your current openclaw.json</p>
            </div>
          </div>
          <div className="bg-dark-600 rounded-xl p-4 mb-4 flex items-center gap-3">
            <FileJson size={20} className="text-gray-400 flex-shrink-0" />
            <div>
              <p className="text-sm text-gray-300">openclaw.json</p>
              <p className="text-xs text-gray-500">Includes AI providers, channels, agents, MCP servers, and all settings</p>
            </div>
          </div>
          <button
            onClick={handleExport}
            disabled={exportLoading}
            className="btn-primary flex items-center gap-2"
          >
            {exportLoading ? <Loader2 size={16} className="animate-spin" /> : <Download size={16} />}
            Export Now
          </button>
        </div>

        {/* Import / Restore card */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center">
              <Upload size={20} className="text-amber-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">Restore Configuration</h3>
              <p className="text-xs text-gray-500">Import a previously exported backup file</p>
            </div>
          </div>

          <div className="bg-amber-500/10 border border-amber-500/20 rounded-xl p-3 mb-4 flex items-start gap-2">
            <AlertCircle size={16} className="text-amber-400 mt-0.5 flex-shrink-0" />
            <p className="text-xs text-amber-300">
              Restoring will overwrite your current configuration. Make sure to export a backup first.
            </p>
          </div>

          <label className={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-all cursor-pointer w-fit
            ${importLoading ? 'opacity-50 pointer-events-none' : 'bg-amber-500/20 text-amber-300 border border-amber-500/30 hover:bg-amber-500/30'}`}>
            {importLoading ? <Loader2 size={16} className="animate-spin" /> : <Upload size={16} />}
            Choose Backup File
            <input type="file" accept=".json" onChange={handleImportFile} className="hidden" disabled={importLoading} />
          </label>
        </div>

        {/* Security note */}
        <div className="bg-dark-700/50 rounded-xl p-4 border border-dark-500 flex items-start gap-3">
          <ShieldCheck size={18} className="text-green-400 mt-0.5 flex-shrink-0" />
          <div>
            <p className="text-sm font-medium text-gray-300">Security Note</p>
            <p className="text-xs text-gray-500 mt-1">
              Backup files may contain API keys and tokens. Store them securely and never share them publicly.
            </p>
          </div>
        </div>
      </div>

      {/* Restore confirmation modal */}
      {showRestoreConfirm && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <motion.div
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            className="bg-dark-800 border border-dark-500 rounded-2xl p-6 max-w-lg w-full shadow-2xl"
          >
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center">
                <RotateCcw size={20} className="text-amber-400" />
              </div>
              <div>
                <h3 className="text-lg font-bold text-white">Confirm Restore</h3>
                <p className="text-xs text-gray-400">This will overwrite your current configuration</p>
              </div>
            </div>

            {jsonPreview && (
              <div className="bg-dark-900 rounded-xl p-3 mb-4 max-h-48 overflow-y-auto">
                <pre className="text-xs text-gray-400 font-mono whitespace-pre-wrap break-all">
                  {jsonPreview.slice(0, 800)}{jsonPreview.length > 800 ? '\n...' : ''}
                </pre>
              </div>
            )}

            <div className="flex gap-3 justify-end">
              <button
                onClick={() => { setShowRestoreConfirm(false); setPendingRestore(null); setJsonPreview(null); }}
                className="px-4 py-2 text-gray-300 hover:text-white hover:bg-dark-700 rounded-lg transition-colors text-sm"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmRestore}
                className="px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-lg transition-colors text-sm flex items-center gap-2"
              >
                <RotateCcw size={14} />
                Restore
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </div>
  );
}
