// Dictation Panel Component for voice-to-text input
// Integrates with DocumentEditor for seamless dictation

import React, { useEffect } from 'react';
import { useDictation } from '../hooks/useDictation';
import { Mic, MicOff, StopCircle, RotateCcw, AlertCircle } from 'lucide-react';

interface DictationPanelProps {
  onTranscript: (text: string) => void;
  className?: string;
}

export function DictationPanel({ onTranscript, className = '' }: DictationPanelProps) {
  const {
    isListening,
    transcript,
    interimTranscript,
    error,
    isSupported,
    start,
    stop,
    toggle,
    reset,
    clearError,
  } = useDictation({
    continuous: true,
    interimResults: true,
    legalMode: true,
    onFinalTranscript: (text) => {
      onTranscript(text);
    },
  });

  // Auto-dismiss errors after 5 seconds
  useEffect(() => {
    if (error) {
      const timer = setTimeout(clearError, 5000);
      return () => clearTimeout(timer);
    }
  }, [error, clearError]);

  if (!isSupported) {
    return (
      <div className={`p-4 bg-yellow-50 border border-yellow-200 rounded-lg ${className}`}>
        <div className="flex items-center gap-2 text-yellow-800">
          <AlertCircle className="w-5 h-5" />
          <div>
            <p className="font-semibold">Voice dictation not supported</p>
            <p className="text-sm">Please use Chrome, Edge, or Safari for voice dictation.</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-lg shadow-sm ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-gray-900">Voice Dictation</h3>
        <div className="flex items-center gap-2">
          {isListening && (
            <span className="flex items-center gap-2 text-sm text-green-600">
              <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></span>
              Listening
            </span>
          )}
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={toggle}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors ${
            isListening
              ? 'bg-red-500 text-white hover:bg-red-600'
              : 'bg-blue-500 text-white hover:bg-blue-600'
          }`}
        >
          {isListening ? (
            <>
              <StopCircle className="w-5 h-5" />
              Stop Dictating
            </>
          ) : (
            <>
              <Mic className="w-5 h-5" />
              Start Dictating
            </>
          )}
        </button>

        <button
          type="button"
          onClick={reset}
          disabled={!transcript && !interimTranscript}
          className="flex items-center gap-2 px-4 py-2 rounded-lg font-medium border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <RotateCcw className="w-4 h-4" />
          Reset
        </button>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
          <div className="flex items-center gap-2 text-red-800">
            <AlertCircle className="w-4 h-4" />
            <p className="text-sm font-medium">Error: {error}</p>
          </div>
        </div>
      )}

      {/* Transcript Display */}
      <div className="flex flex-col gap-2">
        <label className="text-sm font-medium text-gray-700">Current Transcript</label>
        <div className="p-3 bg-gray-50 border border-gray-200 rounded-lg min-h-[120px] max-h-[300px] overflow-y-auto">
          <p className="text-sm text-gray-900 whitespace-pre-wrap">
            {transcript}
            {interimTranscript && (
              <span className="text-gray-400 italic">{interimTranscript}</span>
            )}
          </p>
          {!transcript && !interimTranscript && (
            <p className="text-sm text-gray-400 italic">
              Click "Start Dictating" and begin speaking...
            </p>
          )}
        </div>
      </div>

      {/* Voice Commands Help */}
      <details className="group">
        <summary className="text-sm font-medium text-gray-700 cursor-pointer hover:text-gray-900">
          Voice Commands
        </summary>
        <div className="mt-2 p-3 bg-gray-50 rounded-lg text-sm text-gray-600 space-y-2">
          <div>
            <p className="font-semibold text-gray-700">Punctuation:</p>
            <ul className="ml-4 list-disc space-y-1">
              <li>"period", "comma", "question mark", "exclamation point"</li>
              <li>"colon", "semicolon", "dash"</li>
              <li>"open quote", "close quote"</li>
              <li>"open parenthesis", "close parenthesis"</li>
            </ul>
          </div>
          <div>
            <p className="font-semibold text-gray-700">Formatting:</p>
            <ul className="ml-4 list-disc space-y-1">
              <li>"new paragraph" - Insert paragraph break</li>
              <li>"new line" - Insert line break</li>
              <li>"capitalize" - Capitalize next word</li>
              <li>"all caps" - Make next word all caps</li>
            </ul>
          </div>
          <div>
            <p className="font-semibold text-gray-700">Editing:</p>
            <ul className="ml-4 list-disc space-y-1">
              <li>"scratch that" / "undo that" - Delete last text</li>
              <li>"delete last word" - Remove last word</li>
              <li>"delete last sentence" - Remove last sentence</li>
            </ul>
          </div>
          <div>
            <p className="font-semibold text-gray-700">Legal Documents:</p>
            <ul className="ml-4 list-disc space-y-1">
              <li>"insert citation" - Add citation placeholder</li>
              <li>"insert date" - Insert today's date</li>
              <li>"insert signature block" - Add signature section</li>
              <li>"insert caption" - Add case caption</li>
            </ul>
          </div>
          <div>
            <p className="font-semibold text-gray-700 mt-2">Auto-Corrections:</p>
            <p className="text-xs">
              Legal terms are automatically capitalized and formatted correctly
              (e.g., "plaintiff" → "Plaintiff", "versus" → "v.").
            </p>
          </div>
        </div>
      </details>
    </div>
  );
}
