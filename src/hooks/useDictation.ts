// Voice-to-text dictation hook using Web Speech API
// Provides continuous dictation with legal command support

import { useState, useEffect, useRef, useCallback } from 'react';

interface DictationState {
  isListening: boolean;
  transcript: string;
  interimTranscript: string;
  error: string | null;
  isSupported: boolean;
}

interface DictationCommands {
  'new paragraph': () => void;
  'new line': () => void;
  'period': () => void;
  'comma': () => void;
  'question mark': () => void;
  'exclamation point': () => void;
  'colon': () => void;
  'semicolon': () => void;
  'open quote': () => void;
  'close quote': () => void;
  'open parenthesis': () => void;
  'close parenthesis': () => void;
  'dash': () => void;
  'underscore': () => void;
  'capitalize': () => void;
  'all caps': () => void;
  'undo that': () => void;
  'delete': () => void;
  'delete last word': () => void;
  'delete last sentence': () => void;
  'scratch that': () => void;
  'insert citation': () => void;
  'insert date': () => void;
  'insert signature block': () => void;
  'insert caption': () => void;
}

interface UseDictationOptions {
  language?: string;
  continuous?: boolean;
  interimResults?: boolean;
  maxAlternatives?: number;
  onTranscript?: (transcript: string) => void;
  onFinalTranscript?: (transcript: string) => void;
  onCommand?: (command: string) => void;
  legalMode?: boolean;
}

export function useDictation(options: UseDictationOptions = {}) {
  const {
    language = 'en-US',
    continuous = true,
    interimResults = true,
    maxAlternatives = 1,
    onTranscript,
    onFinalTranscript,
    onCommand,
    legalMode = true,
  } = options;

  const [state, setState] = useState<DictationState>({
    isListening: false,
    transcript: '',
    interimTranscript: '',
    error: null,
    isSupported: false,
  });

  const recognitionRef = useRef<SpeechRecognition | null>(null);
  const finalTranscriptRef = useRef('');

  // Check for browser support
  useEffect(() => {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    const isSupported = !!SpeechRecognition;

    setState(prev => ({ ...prev, isSupported }));

    if (isSupported) {
      const recognition = new SpeechRecognition();
      recognition.lang = language;
      recognition.continuous = continuous;
      recognition.interimResults = interimResults;
      recognition.maxAlternatives = maxAlternatives;

      recognitionRef.current = recognition;

      // Handle results
      recognition.onresult = (event: SpeechRecognitionEvent) => {
        let interim = '';
        let final = '';

        for (let i = event.resultIndex; i < event.results.length; i++) {
          const transcript = event.results[i][0].transcript;
          if (event.results[i].isFinal) {
            final += transcript;
          } else {
            interim += transcript;
          }
        }

        if (final) {
          // Check for voice commands
          const processedText = legalMode
            ? processLegalCommands(final, finalTranscriptRef.current)
            : final;

          finalTranscriptRef.current += processedText;

          setState(prev => ({
            ...prev,
            transcript: finalTranscriptRef.current,
            interimTranscript: '',
          }));

          onFinalTranscript?.(processedText);
          onTranscript?.(finalTranscriptRef.current);
        } else if (interim) {
          setState(prev => ({
            ...prev,
            interimTranscript: interim,
          }));
        }
      };

      // Handle errors
      recognition.onerror = (event: SpeechRecognitionErrorEvent) => {
        console.error('Speech recognition error:', event.error);
        setState(prev => ({
          ...prev,
          error: event.error,
          isListening: false,
        }));
      };

      // Handle end
      recognition.onend = () => {
        setState(prev => ({
          ...prev,
          isListening: false,
        }));
      };
    }

    return () => {
      if (recognitionRef.current) {
        recognitionRef.current.stop();
      }
    };
  }, [language, continuous, interimResults, maxAlternatives, legalMode]);

  // Start dictation
  const start = useCallback(() => {
    if (recognitionRef.current && !state.isListening) {
      try {
        recognitionRef.current.start();
        setState(prev => ({
          ...prev,
          isListening: true,
          error: null,
        }));
      } catch (error) {
        console.error('Failed to start dictation:', error);
        setState(prev => ({
          ...prev,
          error: 'Failed to start dictation',
        }));
      }
    }
  }, [state.isListening]);

  // Stop dictation
  const stop = useCallback(() => {
    if (recognitionRef.current && state.isListening) {
      recognitionRef.current.stop();
      setState(prev => ({
        ...prev,
        isListening: false,
      }));
    }
  }, [state.isListening]);

  // Toggle dictation
  const toggle = useCallback(() => {
    if (state.isListening) {
      stop();
    } else {
      start();
    }
  }, [state.isListening, start, stop]);

  // Reset transcript
  const reset = useCallback(() => {
    finalTranscriptRef.current = '';
    setState(prev => ({
      ...prev,
      transcript: '',
      interimTranscript: '',
      error: null,
    }));
  }, []);

  // Clear error
  const clearError = useCallback(() => {
    setState(prev => ({
      ...prev,
      error: null,
    }));
  }, []);

  return {
    ...state,
    start,
    stop,
    toggle,
    reset,
    clearError,
  };
}

// Process legal voice commands
function processLegalCommands(text: string, context: string): string {
  let processed = text.toLowerCase().trim();

  // Punctuation commands
  if (processed === 'new paragraph') {
    return '\n\n';
  }
  if (processed === 'new line') {
    return '\n';
  }
  if (processed === 'period') {
    return '. ';
  }
  if (processed === 'comma') {
    return ', ';
  }
  if (processed === 'question mark') {
    return '? ';
  }
  if (processed === 'exclamation point') {
    return '! ';
  }
  if (processed === 'colon') {
    return ': ';
  }
  if (processed === 'semicolon') {
    return '; ';
  }
  if (processed === 'dash' || processed === 'hyphen') {
    return '-';
  }
  if (processed === 'underscore') {
    return '_';
  }

  // Quote commands
  if (processed === 'open quote' || processed === 'quote') {
    return '"';
  }
  if (processed === 'close quote' || processed === 'end quote') {
    return '" ';
  }

  // Parenthesis commands
  if (processed === 'open parenthesis' || processed === 'open paren') {
    return '(';
  }
  if (processed === 'close parenthesis' || processed === 'close paren') {
    return ') ';
  }

  // Deletion commands
  if (processed === 'scratch that' || processed === 'undo that' || processed === 'delete') {
    return '[DELETE_LAST]';
  }
  if (processed === 'delete last word') {
    return '[DELETE_LAST_WORD]';
  }
  if (processed === 'delete last sentence') {
    return '[DELETE_LAST_SENTENCE]';
  }

  // Formatting commands
  if (processed === 'capitalize') {
    return '[CAPITALIZE_NEXT]';
  }
  if (processed === 'all caps') {
    return '[CAPS_NEXT]';
  }

  // Legal-specific commands
  if (processed === 'insert citation' || processed === 'cite') {
    return '[INSERT_CITATION]';
  }
  if (processed === 'insert date') {
    const today = new Date().toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
    return today;
  }
  if (processed === 'insert signature block') {
    return '[INSERT_SIGNATURE]';
  }
  if (processed === 'insert caption') {
    return '[INSERT_CAPTION]';
  }

  // Legal terms auto-correction
  processed = autoCorrectLegalTerms(text);

  // Smart capitalization for legal documents
  if (context.endsWith('. ') || context.endsWith('.\n') || context.length === 0) {
    processed = capitalize(processed);
  }

  return processed + ' ';
}

// Auto-correct common legal terms
function autoCorrectLegalTerms(text: string): string {
  const corrections: Record<string, string> = {
    'plaintiff': 'Plaintiff',
    'defendant': 'Defendant',
    'plaintiffs': 'Plaintiffs',
    'defendants': 'Defendants',
    'appellant': 'Appellant',
    'appellee': 'Appellee',
    'petitioner': 'Petitioner',
    'respondent': 'Respondent',
    'commonwealth': 'Commonwealth',
    'honorable': 'Honorable',
    'esquire': 'Esquire',
    'versus': 'v.',
    'vs': 'v.',
    'ex parte': 'ex parte',
    'in re': 'In re',
    'pro se': 'pro se',
    'per curiam': 'per curiam',
    'habeas corpus': 'habeas corpus',
    'amicus curiae': 'amicus curiae',
    'prima facie': 'prima facie',
    'res judicata': 'res judicata',
    'stare decisis': 'stare decisis',
    'voir dire': 'voir dire',
    'subpoena': 'subpoena',
    'affidavit': 'affidavit',
    'deposition': 'deposition',
    'interrogatories': 'interrogatories',
    'wherefore': 'WHEREFORE',
    'comes now': 'COMES NOW',
  };

  let result = text;
  for (const [wrong, correct] of Object.entries(corrections)) {
    const regex = new RegExp(`\\b${wrong}\\b`, 'gi');
    result = result.replace(regex, correct);
  }

  return result;
}

function capitalize(text: string): string {
  if (!text) return text;
  return text.charAt(0).toUpperCase() + text.slice(1);
}

// TypeScript declarations for Web Speech API
declare global {
  interface Window {
    SpeechRecognition: typeof SpeechRecognition;
    webkitSpeechRecognition: typeof SpeechRecognition;
  }

  interface SpeechRecognition extends EventTarget {
    lang: string;
    continuous: boolean;
    interimResults: boolean;
    maxAlternatives: number;
    start(): void;
    stop(): void;
    abort(): void;
    onresult: ((event: SpeechRecognitionEvent) => void) | null;
    onerror: ((event: SpeechRecognitionErrorEvent) => void) | null;
    onend: (() => void) | null;
  }

  const SpeechRecognition: {
    prototype: SpeechRecognition;
    new (): SpeechRecognition;
  };

  interface SpeechRecognitionEvent extends Event {
    resultIndex: number;
    results: SpeechRecognitionResultList;
  }

  interface SpeechRecognitionResultList {
    length: number;
    item(index: number): SpeechRecognitionResult;
    [index: number]: SpeechRecognitionResult;
  }

  interface SpeechRecognitionResult {
    length: number;
    item(index: number): SpeechRecognitionAlternative;
    [index: number]: SpeechRecognitionAlternative;
    isFinal: boolean;
  }

  interface SpeechRecognitionAlternative {
    transcript: string;
    confidence: number;
  }

  interface SpeechRecognitionErrorEvent extends Event {
    error: string;
    message: string;
  }
}
