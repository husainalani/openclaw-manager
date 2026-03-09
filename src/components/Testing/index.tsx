import { useState } from 'react';
import { motion } from 'framer-motion';
import { api, isTauri, DiagnosticResult, AITestResult } from '../../lib/tauri';
import {
  CheckCircle,
  XCircle,
  Play,
  Loader2,
  Stethoscope,
  Bot,
  MessageSquare,
  Clock,
  Zap,
} from 'lucide-react';
import clsx from 'clsx';
import { testingLogger } from '../../lib/logger';

interface DiagnosticRun {
  timestamp: Date;
  results: DiagnosticResult[];
  passed: number;
  failed: number;
}

const CHANNEL_TYPES = ['telegram', 'discord', 'slack', 'webhook'];

export function Testing() {
  // Diagnostics
  const [diagnosticResults, setDiagnosticResults] = useState<DiagnosticResult[]>([]);
  const [diagnosticLoading, setDiagnosticLoading] = useState(false);
  const [diagnosticHistory, setDiagnosticHistory] = useState<DiagnosticRun[]>([]);

  // AI test
  const [aiTestResult, setAiTestResult] = useState<AITestResult | null>(null);
  const [aiTestLoading, setAiTestLoading] = useState(false);
  const [aiTestError, setAiTestError] = useState<string | null>(null);

  // Channel test
  const [channelType, setChannelType] = useState(CHANNEL_TYPES[0]);
  const [channelTestResult, setChannelTestResult] = useState<string | null>(null);
  const [channelTestSuccess, setChannelTestSuccess] = useState<boolean | null>(null);
  const [channelTestLoading, setChannelTestLoading] = useState(false);

  const runDiagnostics = async () => {
    testingLogger.action('Run system diagnostics');
    setDiagnosticLoading(true);
    setDiagnosticResults([]);
    try {
      const results = await api.runDoctor();
      setDiagnosticResults(results);
      const passed = results.filter(r => r.passed).length;
      const run: DiagnosticRun = { timestamp: new Date(), results, passed, failed: results.length - passed };
      setDiagnosticHistory(prev => [run, ...prev].slice(0, 5)); // keep last 5
      testingLogger.info(`Diagnostics completed: ${passed}/${results.length} passed`);
    } catch (e) {
      testingLogger.error('Diagnostics failed', e);
      setDiagnosticResults([{
        name: 'Diagnostics Execution',
        passed: false,
        message: String(e),
        suggestion: 'Please check if OpenClaw is properly installed',
      }]);
    } finally {
      setDiagnosticLoading(false);
    }
  };

  const runAITest = async () => {
    if (!isTauri()) return;
    testingLogger.action('Test AI connection');
    setAiTestLoading(true);
    setAiTestResult(null);
    setAiTestError(null);
    try {
      const result = await api.testAIConnection();
      setAiTestResult(result);
      testingLogger.info('AI test completed', result);
    } catch (e) {
      setAiTestError(String(e));
      testingLogger.error('AI test failed', e);
    } finally {
      setAiTestLoading(false);
    }
  };

  const runChannelTest = async () => {
    if (!isTauri()) return;
    testingLogger.action(`Test channel: ${channelType}`);
    setChannelTestLoading(true);
    setChannelTestResult(null);
    setChannelTestSuccess(null);
    try {
      const result = await api.testChannel(channelType);
      setChannelTestResult(typeof result === 'string' ? result : JSON.stringify(result));
      setChannelTestSuccess(true);
      testingLogger.info(`Channel ${channelType} test passed`);
    } catch (e) {
      setChannelTestResult(String(e));
      setChannelTestSuccess(false);
      testingLogger.error(`Channel ${channelType} test failed`, e);
    } finally {
      setChannelTestLoading(false);
    }
  };

  const passedCount = diagnosticResults.filter(r => r.passed).length;
  const failedCount = diagnosticResults.filter(r => !r.passed).length;

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2">
      <div className="max-w-4xl space-y-6">

        {/* ── System Diagnostics ── */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
                <Stethoscope size={20} className="text-purple-400" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-white">System Diagnostics</h3>
                <p className="text-xs text-gray-500">Check OpenClaw installation and configuration status</p>
              </div>
            </div>
            <button onClick={runDiagnostics} disabled={diagnosticLoading} className="btn-primary flex items-center gap-2">
              {diagnosticLoading ? <Loader2 size={16} className="animate-spin" /> : <Play size={16} />}
              Run Diagnostics
            </button>
          </div>

          {/* Summary */}
          {diagnosticResults.length > 0 && (
            <div className="flex gap-4 mb-4 p-3 bg-dark-600 rounded-lg">
              <div className="flex items-center gap-2">
                <CheckCircle size={16} className="text-green-400" />
                <span className="text-sm text-green-400">{passedCount} passed</span>
              </div>
              {failedCount > 0 && (
                <div className="flex items-center gap-2">
                  <XCircle size={16} className="text-red-400" />
                  <span className="text-sm text-red-400">{failedCount} failed</span>
                </div>
              )}
            </div>
          )}

          {/* Results */}
          {diagnosticResults.length > 0 && (
            <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="space-y-2">
              {diagnosticResults.map((result, index) => (
                <div key={index} className={clsx('flex items-start gap-3 p-3 rounded-lg', result.passed ? 'bg-green-500/10' : 'bg-red-500/10')}>
                  {result.passed
                    ? <CheckCircle size={18} className="text-green-400 mt-0.5 flex-shrink-0" />
                    : <XCircle size={18} className="text-red-400 mt-0.5 flex-shrink-0" />}
                  <div className="flex-1 min-w-0">
                    <p className={clsx('text-sm font-medium', result.passed ? 'text-green-400' : 'text-red-400')}>{result.name}</p>
                    <p className="text-xs text-gray-400 mt-1 whitespace-pre-wrap break-words">{result.message}</p>
                    {result.suggestion && <p className="text-xs text-amber-400 mt-1">💡 {result.suggestion}</p>}
                  </div>
                </div>
              ))}
            </motion.div>
          )}

          {diagnosticResults.length === 0 && !diagnosticLoading && (
            <div className="text-center py-8 text-gray-500">
              <Stethoscope size={48} className="mx-auto mb-3 opacity-30" />
              <p>Click "Run Diagnostics" to start checking system status</p>
            </div>
          )}

          {/* History */}
          {diagnosticHistory.length > 0 && (
            <div className="mt-4 border-t border-dark-500 pt-4">
              <p className="text-xs text-gray-500 mb-2 flex items-center gap-1"><Clock size={12} /> Recent runs</p>
              <div className="flex gap-2 flex-wrap">
                {diagnosticHistory.map((run, i) => (
                  <button
                    key={i}
                    onClick={() => setDiagnosticResults(run.results)}
                    className={clsx(
                      'text-xs px-3 py-1.5 rounded-lg border transition-colors',
                      run.failed === 0
                        ? 'border-green-500/30 text-green-400 bg-green-500/5 hover:bg-green-500/10'
                        : 'border-red-500/30 text-red-400 bg-red-500/5 hover:bg-red-500/10'
                    )}
                  >
                    {run.timestamp.toLocaleTimeString()} — {run.passed}/{run.passed + run.failed} passed
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* ── AI Connection Test ── */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-pink-500/20 flex items-center justify-center">
                <Bot size={20} className="text-pink-400" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-white">AI Connection Test</h3>
                <p className="text-xs text-gray-500">Test your configured AI provider and primary model</p>
              </div>
            </div>
            <button onClick={runAITest} disabled={aiTestLoading} className="btn-primary flex items-center gap-2">
              {aiTestLoading ? <Loader2 size={16} className="animate-spin" /> : <Zap size={16} />}
              Test AI
            </button>
          </div>

          {aiTestError && (
            <div className="flex items-start gap-3 p-3 rounded-lg bg-red-500/10">
              <XCircle size={18} className="text-red-400 mt-0.5 flex-shrink-0" />
              <p className="text-sm text-red-300 break-all">{aiTestError}</p>
            </div>
          )}

          {aiTestResult && (
            <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className={clsx('p-4 rounded-xl', aiTestResult.success ? 'bg-green-500/10 border border-green-500/30' : 'bg-red-500/10 border border-red-500/30')}>
              <div className="flex items-center gap-2 mb-3">
                {aiTestResult.success
                  ? <CheckCircle size={18} className="text-green-400" />
                  : <XCircle size={18} className="text-red-400" />}
                <span className={clsx('text-sm font-medium', aiTestResult.success ? 'text-green-400' : 'text-red-400')}>
                  {aiTestResult.success ? 'Connection successful' : 'Connection failed'}
                </span>
                {aiTestResult.latency_ms && (
                  <span className="ml-auto text-xs text-gray-500 flex items-center gap-1">
                    <Zap size={11} /> {aiTestResult.latency_ms}ms
                  </span>
                )}
              </div>
              <div className="grid grid-cols-2 gap-2 text-xs mb-3">
                <div className="bg-dark-600 rounded-lg p-2">
                  <span className="text-gray-500">Provider</span>
                  <p className="text-white font-mono mt-0.5">{aiTestResult.provider || '—'}</p>
                </div>
                <div className="bg-dark-600 rounded-lg p-2">
                  <span className="text-gray-500">Model</span>
                  <p className="text-white font-mono mt-0.5">{aiTestResult.model || '—'}</p>
                </div>
              </div>
              {aiTestResult.response && (
                <div className="bg-dark-600 rounded-lg p-3">
                  <p className="text-xs text-gray-500 mb-1">Response</p>
                  <p className="text-xs text-gray-300 break-words">{aiTestResult.response}</p>
                </div>
              )}
              {aiTestResult.error && (
                <p className="text-xs text-red-300 mt-2">{aiTestResult.error}</p>
              )}
            </motion.div>
          )}

          {!aiTestResult && !aiTestError && !aiTestLoading && (
            <div className="text-center py-6 text-gray-500">
              <Bot size={36} className="mx-auto mb-2 opacity-30" />
              <p className="text-sm">Click "Test AI" to verify your AI provider connection</p>
            </div>
          )}
        </div>

        {/* ── Channel Test ── */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
                <MessageSquare size={20} className="text-blue-400" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-white">Channel Test</h3>
                <p className="text-xs text-gray-500">Test connectivity for a specific message channel</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <select
                value={channelType}
                onChange={(e) => { setChannelType(e.target.value); setChannelTestResult(null); setChannelTestSuccess(null); }}
                className="bg-dark-600 border border-dark-500 rounded-lg px-3 py-2 text-sm text-gray-300 capitalize"
              >
                {CHANNEL_TYPES.map(t => (
                  <option key={t} value={t}>{t.charAt(0).toUpperCase() + t.slice(1)}</option>
                ))}
              </select>
              <button onClick={runChannelTest} disabled={channelTestLoading} className="btn-primary flex items-center gap-2">
                {channelTestLoading ? <Loader2 size={16} className="animate-spin" /> : <Play size={16} />}
                Test
              </button>
            </div>
          </div>

          {channelTestResult && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              className={clsx('flex items-start gap-3 p-3 rounded-lg', channelTestSuccess ? 'bg-green-500/10' : 'bg-red-500/10')}
            >
              {channelTestSuccess
                ? <CheckCircle size={18} className="text-green-400 mt-0.5 flex-shrink-0" />
                : <XCircle size={18} className="text-red-400 mt-0.5 flex-shrink-0" />}
              <p className={clsx('text-sm break-all', channelTestSuccess ? 'text-green-300' : 'text-red-300')}>
                {channelTestResult}
              </p>
            </motion.div>
          )}

          {!channelTestResult && !channelTestLoading && (
            <div className="text-center py-6 text-gray-500">
              <MessageSquare size={36} className="mx-auto mb-2 opacity-30" />
              <p className="text-sm">Select a channel type and click "Test" to check connectivity</p>
            </div>
          )}
        </div>

        {/* Instructions */}
        <div className="bg-dark-700/50 rounded-xl p-4 border border-dark-500">
          <h4 className="text-sm font-medium text-gray-400 mb-2">Instructions</h4>
          <ul className="text-sm text-gray-500 space-y-1">
            <li>• System diagnostics checks Node.js, OpenClaw installation, config files and other status</li>
            <li>• AI test requires a configured provider in <span className="text-claw-400">AI Configuration</span></li>
            <li>• Channel test requires the channel to be configured in <span className="text-claw-400">Message Channels</span></li>
          </ul>
        </div>
      </div>
    </div>
  );
}
