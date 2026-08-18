import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './AudioMergeDialog.css';

interface AudioMergeDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onMergeComplete: (outputPath: string) => void;
}

interface AudioFile {
  id: string;
  path: string;
  name: string;
  duration?: number;
}

interface AudioOperationResult {
  success: boolean;
  output_path?: string;
  error?: string;
}

const AudioMergeDialog: React.FC<AudioMergeDialogProps> = ({
  isOpen,
  onClose,
  onMergeComplete,
}) => {
  const [selectedFiles, setSelectedFiles] = useState<AudioFile[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFileSelect = async () => {
    try {
      // Use Tauri dialog API to select files
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: 'Audio',
            extensions: ['m4a', 'mp3', 'wav', 'aac', 'ogg', 'flac'],
          },
        ],
      });

      if (selected && Array.isArray(selected)) {
        const files: AudioFile[] = selected.map((path, index) => ({
          id: `file-${Date.now()}-${index}`,
          path: path as string,
          name: (path as string).split('/').pop() || 'Unknown',
        }));

        // Get durations for each file
        const filesWithDuration = await Promise.all(
          files.map(async (file) => {
            try {
              const duration = await invoke<number>('get_audio_duration', {
                filePath: file.path,
              });
              return { ...file, duration };
            } catch (err) {
              console.error('Failed to get duration:', err);
              return file;
            }
          })
        );

        setSelectedFiles([...selectedFiles, ...filesWithDuration]);
        setError(null);
      }
    } catch (err) {
      console.error('Failed to select files:', err);
      setError('Failed to select files');
    }
  };

  const handleRemoveFile = (id: string) => {
    setSelectedFiles(selectedFiles.filter((f) => f.id !== id));
  };

  const handleMoveUp = (index: number) => {
    if (index === 0) return;
    const newFiles = [...selectedFiles];
    [newFiles[index - 1], newFiles[index]] = [newFiles[index], newFiles[index - 1]];
    setSelectedFiles(newFiles);
  };

  const handleMoveDown = (index: number) => {
    if (index === selectedFiles.length - 1) return;
    const newFiles = [...selectedFiles];
    [newFiles[index], newFiles[index + 1]] = [newFiles[index + 1], newFiles[index]];
    setSelectedFiles(newFiles);
  };

  const handleMerge = async () => {
    if (selectedFiles.length < 2) {
      setError('Please select at least 2 files to merge');
      return;
    }

    setIsProcessing(true);
    setError(null);

    try {
      const result = await invoke<AudioOperationResult>('merge_audio_files', {
        request: {
          file_paths: selectedFiles.map((f) => f.path),
        },
      });

      if (result.success && result.output_path) {
        onMergeComplete(result.output_path);
        handleClose();
      } else {
        setError(result.error || 'Failed to merge audio files');
      }
    } catch (err) {
      console.error('Merge failed:', err);
      setError(`Failed to merge audio: ${err}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleClose = () => {
    setSelectedFiles([]);
    setError(null);
    setIsProcessing(false);
    onClose();
  };

  const formatDuration = (ms?: number): string => {
    if (!ms) return '--:--';
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  const getTotalDuration = (): number => {
    return selectedFiles.reduce((sum, file) => sum + (file.duration || 0), 0);
  };

  if (!isOpen) return null;

  return (
    <div className="audio-merge-dialog-overlay" onClick={handleClose}>
      <div className="audio-merge-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Merge Audio Files</h2>
          <button className="close-button" onClick={handleClose} aria-label="Close">
            ×
          </button>
        </div>

        <div className="dialog-content">
          <div className="file-selection-area">
            <button
              className="select-files-button"
              onClick={handleFileSelect}
              disabled={isProcessing}
            >
              + Select Audio Files
            </button>

            {selectedFiles.length > 0 && (
              <div className="selected-files-list">
                <h3>Selected Files ({selectedFiles.length})</h3>
                {selectedFiles.map((file, index) => (
                  <div key={file.id} className="file-item">
                    <div className="file-order">{index + 1}</div>
                    <div className="file-info">
                      <div className="file-name">{file.name}</div>
                      <div className="file-duration">{formatDuration(file.duration)}</div>
                    </div>
                    <div className="file-actions">
                      <button
                        onClick={() => handleMoveUp(index)}
                        disabled={index === 0 || isProcessing}
                        aria-label="Move up"
                      >
                        ↑
                      </button>
                      <button
                        onClick={() => handleMoveDown(index)}
                        disabled={index === selectedFiles.length - 1 || isProcessing}
                        aria-label="Move down"
                      >
                        ↓
                      </button>
                      <button
                        onClick={() => handleRemoveFile(file.id)}
                        disabled={isProcessing}
                        className="remove-button"
                        aria-label="Remove"
                      >
                        ×
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {selectedFiles.length > 0 && (
            <div className="merge-summary">
              <div className="summary-item">
                <span>Total files:</span>
                <strong>{selectedFiles.length}</strong>
              </div>
              <div className="summary-item">
                <span>Combined duration:</span>
                <strong>{formatDuration(getTotalDuration())}</strong>
              </div>
            </div>
          )}

          {error && <div className="error-message">{error}</div>}
        </div>

        <div className="dialog-footer">
          <button className="cancel-button" onClick={handleClose} disabled={isProcessing}>
            Cancel
          </button>
          <button
            className="merge-button"
            onClick={handleMerge}
            disabled={selectedFiles.length < 2 || isProcessing}
          >
            {isProcessing ? 'Merging...' : 'Merge Files'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default AudioMergeDialog;
