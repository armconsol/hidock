import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './AudioReplaceDialog.css';

interface AudioReplaceDialogProps {
  isOpen: boolean;
  onClose: () => void;
  originalFilePath: string;
  onReplaceComplete: (outputPath: string) => void;
}

interface AudioOperationResult {
  success: boolean;
  output_path?: string;
  error?: string;
}

const AudioReplaceDialog: React.FC<AudioReplaceDialogProps> = ({
  isOpen,
  onClose,
  originalFilePath,
  onReplaceComplete,
}) => {
  const [replacementFilePath, setReplacementFilePath] = useState<string | null>(null);
  const [replacementFileName, setReplacementFileName] = useState<string>('');
  const [startTime, setStartTime] = useState<number>(0);
  const [endTime, setEndTime] = useState<number>(0);
  const [originalDuration, setOriginalDuration] = useState<number>(0);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const audioRef = useRef<HTMLAudioElement>(null);

  useEffect(() => {
    if (isOpen && originalFilePath) {
      loadOriginalDuration();
    }
  }, [isOpen, originalFilePath]);

  const loadOriginalDuration = async () => {
    try {
      const duration = await invoke<number>('get_audio_duration', {
        filePath: originalFilePath,
      });
      setOriginalDuration(duration);
      setEndTime(duration);
    } catch (err) {
      console.error('Failed to get duration:', err);
      setError('Failed to load audio duration');
    }
  };

  const handleSelectReplacement = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Audio',
            extensions: ['m4a', 'mp3', 'wav', 'aac', 'ogg', 'flac'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setReplacementFilePath(selected);
        setReplacementFileName(selected.split('/').pop() || 'Unknown');
        setError(null);
      }
    } catch (err) {
      console.error('Failed to select file:', err);
      setError('Failed to select replacement file');
    }
  };

  const handleReplace = async () => {
    if (!replacementFilePath) {
      setError('Please select a replacement audio file');
      return;
    }

    if (startTime >= endTime) {
      setError('Start time must be before end time');
      return;
    }

    if (endTime > originalDuration) {
      setError('End time cannot exceed audio duration');
      return;
    }

    setIsProcessing(true);
    setError(null);

    try {
      const result = await invoke<AudioOperationResult>('replace_audio_segment', {
        request: {
          original_path: originalFilePath,
          replacement_path: replacementFilePath,
          start_ms: startTime,
          end_ms: endTime,
        },
      });

      if (result.success && result.output_path) {
        onReplaceComplete(result.output_path);
        handleClose();
      } else {
        setError(result.error || 'Failed to replace audio segment');
      }
    } catch (err) {
      console.error('Replace failed:', err);
      setError(`Failed to replace audio: ${err}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleClose = () => {
    setReplacementFilePath(null);
    setReplacementFileName('');
    setStartTime(0);
    setEndTime(originalDuration);
    setError(null);
    setIsProcessing(false);
    setIsPlaying(false);
    onClose();
  };

  const formatTime = (ms: number): string => {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${seconds
        .toString()
        .padStart(2, '0')}`;
    }
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  const parseTimeInput = (value: string): number => {
    const parts = value.split(':').map((p) => parseInt(p, 10) || 0);
    if (parts.length === 2) {
      // mm:ss
      return (parts[0] * 60 + parts[1]) * 1000;
    } else if (parts.length === 3) {
      // hh:mm:ss
      return (parts[0] * 3600 + parts[1] * 60 + parts[2]) * 1000;
    }
    return 0;
  };

  const handleStartTimeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const ms = parseTimeInput(e.target.value);
    setStartTime(Math.min(ms, endTime - 1000));
  };

  const handleEndTimeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const ms = parseTimeInput(e.target.value);
    setEndTime(Math.min(ms, originalDuration));
  };

  const togglePlayback = () => {
    if (audioRef.current) {
      if (isPlaying) {
        audioRef.current.pause();
      } else {
        audioRef.current.currentTime = startTime / 1000;
        audioRef.current.play();
      }
      setIsPlaying(!isPlaying);
    }
  };

  const handleAudioEnded = () => {
    setIsPlaying(false);
  };

  if (!isOpen) return null;

  return (
    <div className="audio-replace-dialog-overlay" onClick={handleClose}>
      <div className="audio-replace-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Replace Audio Segment</h2>
          <button className="close-button" onClick={handleClose} aria-label="Close">
            ×
          </button>
        </div>

        <div className="dialog-content">
          <div className="original-audio-info">
            <h3>Original Audio</h3>
            <div className="audio-info">
              <span className="info-label">Duration:</span>
              <span className="info-value">{formatTime(originalDuration)}</span>
            </div>
            <audio
              ref={audioRef}
              src={`asset://localhost/${originalFilePath}`}
              onEnded={handleAudioEnded}
            />
            <button className="playback-button" onClick={togglePlayback}>
              {isPlaying ? '⏸ Pause Preview' : '▶ Preview Selection'}
            </button>
          </div>

          <div className="time-selection">
            <h3>Select Time Range to Replace</h3>
            <div className="time-inputs">
              <div className="time-input-group">
                <label htmlFor="start-time">Start Time</label>
                <input
                  id="start-time"
                  type="text"
                  placeholder="mm:ss"
                  value={formatTime(startTime)}
                  onChange={handleStartTimeChange}
                  disabled={isProcessing}
                />
              </div>
              <div className="time-input-group">
                <label htmlFor="end-time">End Time</label>
                <input
                  id="end-time"
                  type="text"
                  placeholder="mm:ss"
                  value={formatTime(endTime)}
                  onChange={handleEndTimeChange}
                  disabled={isProcessing}
                />
              </div>
            </div>
            <div className="segment-info">
              Segment duration: {formatTime(endTime - startTime)}
            </div>
          </div>

          <div className="replacement-selection">
            <h3>Replacement Audio</h3>
            {replacementFilePath ? (
              <div className="selected-replacement">
                <span className="file-name">{replacementFileName}</span>
                <button
                  onClick={handleSelectReplacement}
                  disabled={isProcessing}
                  className="change-button"
                >
                  Change
                </button>
              </div>
            ) : (
              <button
                className="select-replacement-button"
                onClick={handleSelectReplacement}
                disabled={isProcessing}
              >
                Select Replacement Audio
              </button>
            )}
          </div>

          {error && <div className="error-message">{error}</div>}
        </div>

        <div className="dialog-footer">
          <button className="cancel-button" onClick={handleClose} disabled={isProcessing}>
            Cancel
          </button>
          <button
            className="replace-button"
            onClick={handleReplace}
            disabled={!replacementFilePath || isProcessing}
          >
            {isProcessing ? 'Processing...' : 'Replace Segment'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default AudioReplaceDialog;
