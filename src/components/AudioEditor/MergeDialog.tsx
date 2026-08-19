import { Modal, Button, Typography, Space, Spin } from '@arco-design/web-react';
import { IconUp, IconDown, IconDelete, IconFile } from '@arco-design/web-react/icon';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

const { Text, Title } = Typography;

interface MergeDialogProps {
  visible: boolean;
  onClose: () => void;
  onComplete: (outputPath: string) => void;
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

export function MergeDialog({ visible, onClose, onComplete }: MergeDialogProps) {
  const [selectedFiles, setSelectedFiles] = useState<AudioFile[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFileSelect = async () => {
    try {
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
        onComplete(result.output_path);
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

  return (
    <Modal
      title={<Title heading={5}>Merge Audio Files</Title>}
      visible={visible}
      onCancel={handleClose}
      footer={
        <Space>
          <Button onClick={handleClose} disabled={isProcessing}>
            Cancel
          </Button>
          <Button
            type="primary"
            onClick={handleMerge}
            disabled={selectedFiles.length < 2 || isProcessing}
            loading={isProcessing}
          >
            {isProcessing ? 'Merging...' : 'Merge Files'}
          </Button>
        </Space>
      }
      style={{ width: 600 }}
    >
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        {/* File Selection */}
        <Button type="primary" onClick={handleFileSelect} disabled={isProcessing}>
          Select Audio Files
        </Button>

        {/* Selected Files List */}
        {selectedFiles.length > 0 && (
          <div>
            <Text bold style={{ marginBottom: 10, display: 'block' }}>
              Selected Files ({selectedFiles.length})
            </Text>
            <Space direction="vertical" size="small" style={{ width: '100%' }}>
              {selectedFiles.map((file, index) => (
                <div
                  key={file.id}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    padding: '10px',
                    backgroundColor: '#f7f8fa',
                    borderRadius: '4px',
                  }}
                >
                  <Text style={{ marginRight: 10, fontWeight: 'bold', minWidth: 30 }}>
                    {index + 1}
                  </Text>
                  <IconFile style={{ marginRight: 10 }} />
                  <div style={{ flex: 1 }}>
                    <Text>{file.name}</Text>
                    <br />
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      {formatDuration(file.duration)}
                    </Text>
                  </div>
                  <Space>
                    <Button
                      size="small"
                      icon={<IconUp />}
                      onClick={() => handleMoveUp(index)}
                      disabled={index === 0 || isProcessing}
                    />
                    <Button
                      size="small"
                      icon={<IconDown />}
                      onClick={() => handleMoveDown(index)}
                      disabled={index === selectedFiles.length - 1 || isProcessing}
                    />
                    <Button
                      size="small"
                      icon={<IconDelete />}
                      status="danger"
                      onClick={() => handleRemoveFile(file.id)}
                      disabled={isProcessing}
                    />
                  </Space>
                </div>
              ))}
            </Space>
          </div>
        )}

        {/* Summary */}
        {selectedFiles.length > 0 && (
          <div
            style={{
              padding: '15px',
              backgroundColor: '#e8f3ff',
              borderRadius: '4px',
            }}
          >
            <Space direction="vertical" size="small">
              <div>
                <Text type="secondary">Total files: </Text>
                <Text bold>{selectedFiles.length}</Text>
              </div>
              <div>
                <Text type="secondary">Combined duration: </Text>
                <Text bold>{formatDuration(getTotalDuration())}</Text>
              </div>
            </Space>
          </div>
        )}

        {/* Error Message */}
        {error && (
          <div
            style={{
              padding: '10px',
              backgroundColor: '#ffece8',
              borderRadius: '4px',
            }}
          >
            <Text type="error">{error}</Text>
          </div>
        )}

        {/* Processing Indicator */}
        {isProcessing && (
          <div style={{ textAlign: 'center' }}>
            <Spin />
            <Text style={{ marginLeft: 10 }}>Processing audio files...</Text>
          </div>
        )}
      </Space>
    </Modal>
  );
}
