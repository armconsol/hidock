import { Modal, Button, Typography, Space, InputNumber, Spin } from '@arco-design/web-react';
import { IconFile, IconPlayArrow, IconPause } from '@arco-design/web-react/icon';
import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

const { Text, Title } = Typography;

interface ReplaceDialogProps {
  visible: boolean;
  originalFilePath: string;
  selectedRange?: { start: number; end: number } | null;
  onClose: () => void;
  onComplete: (outputPath: string) => void;
}

interface AudioOperationResult {
  success: boolean;
  output_path?: string;
  error?: string;
}

export function ReplaceDialog({
  visible,
  originalFilePath,
  selectedRange,
  onClose,
  onComplete,
}: ReplaceDialogProps) {
  const audioRef = useRef<HTMLAudioElement>(null);

  const [replacementFilePath, setReplacementFilePath] = useState<string | null>(null);
  const [replacementFileName, setReplacementFileName] = useState<string>('');
  const [startTime, setStartTime] = useState<number>(0);
  const [endTime, setEndTime] = useState<number>(0);
  const [originalDuration, setOriginalDuration] = useState<number>(0);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);

  useEffect(() => {
    if (visible && originalFilePath) {
      loadOriginalDuration();
    }
  }, [visible, originalFilePath]);

  useEffect(() => {
    if (selectedRange) {
      setStartTime(selectedRange.start);
      setEndTime(selectedRange.end);
    }
  }, [selectedRange]);

  const loadOriginalDuration = async () => {
    try {
      const duration = await invoke<number>('get_audio_duration', {
        filePath: originalFilePath,
      });
      setOriginalDuration(duration);
      if (!selectedRange) {
        setEndTime(duration);
      }
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
        onComplete(result.output_path);
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

  return (
    <Modal
      title={<Title heading={5}>Replace Audio Segment</Title>}
      visible={visible}
      onCancel={handleClose}
      footer={
        <Space>
          <Button onClick={handleClose} disabled={isProcessing}>
            Cancel
          </Button>
          <Button
            type="primary"
            onClick={handleReplace}
            disabled={!replacementFilePath || isProcessing}
            loading={isProcessing}
          >
            {isProcessing ? 'Processing...' : 'Replace Segment'}
          </Button>
        </Space>
      }
      style={{ width: 600 }}
    >
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        {/* Original Audio Info */}
        <div>
          <Text bold style={{ marginBottom: 10, display: 'block' }}>
            Original Audio
          </Text>
          <div
            style={{
              padding: '10px',
              backgroundColor: '#f7f8fa',
              borderRadius: '4px',
            }}
          >
            <Text type="secondary">Duration: </Text>
            <Text>{formatTime(originalDuration)}</Text>
            <audio
              ref={audioRef}
              src={`asset://localhost/${originalFilePath}`}
              onEnded={handleAudioEnded}
            />
            <div style={{ marginTop: 10 }}>
              <Button
                size="small"
                icon={isPlaying ? <IconPause /> : <IconPlayArrow />}
                onClick={togglePlayback}
              >
                {isPlaying ? 'Pause Preview' : 'Preview Selection'}
              </Button>
            </div>
          </div>
        </div>

        {/* Time Range Selection */}
        <div>
          <Text bold style={{ marginBottom: 10, display: 'block' }}>
            Select Time Range to Replace
          </Text>
          <Space size="medium">
            <div>
              <Text type="secondary" style={{ display: 'block', marginBottom: 5 }}>
                Start Time (ms)
              </Text>
              <InputNumber
                value={startTime}
                onChange={(value) => setStartTime(value as number)}
                disabled={isProcessing}
                min={0}
                max={endTime - 1000}
                style={{ width: 150 }}
              />
              <br />
              <Text type="secondary" style={{ fontSize: 12 }}>
                {formatTime(startTime)}
              </Text>
            </div>
            <div>
              <Text type="secondary" style={{ display: 'block', marginBottom: 5 }}>
                End Time (ms)
              </Text>
              <InputNumber
                value={endTime}
                onChange={(value) => setEndTime(value as number)}
                disabled={isProcessing}
                min={startTime + 1000}
                max={originalDuration}
                style={{ width: 150 }}
              />
              <br />
              <Text type="secondary" style={{ fontSize: 12 }}>
                {formatTime(endTime)}
              </Text>
            </div>
          </Space>
          <div style={{ marginTop: 10 }}>
            <Text type="secondary">Segment duration: </Text>
            <Text>{formatTime(endTime - startTime)}</Text>
          </div>
        </div>

        {/* Replacement Audio Selection */}
        <div>
          <Text bold style={{ marginBottom: 10, display: 'block' }}>
            Replacement Audio
          </Text>
          {replacementFilePath ? (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                padding: '10px',
                backgroundColor: '#e8f3ff',
                borderRadius: '4px',
              }}
            >
              <IconFile style={{ marginRight: 10 }} />
              <Text style={{ flex: 1 }}>{replacementFileName}</Text>
              <Button
                size="small"
                onClick={handleSelectReplacement}
                disabled={isProcessing}
              >
                Change
              </Button>
            </div>
          ) : (
            <Button
              type="primary"
              onClick={handleSelectReplacement}
              disabled={isProcessing}
            >
              Select Replacement Audio
            </Button>
          )}
        </div>

        {/* Error Message */}
        {error && (
          <div
            style={{
              padding: '10px',
              backgroundColor: '#ffece8',
              borderRadius: '4px',
            }}
          >
            <Text type="danger">{error}</Text>
          </div>
        )}

        {/* Processing Indicator */}
        {isProcessing && (
          <div style={{ textAlign: 'center' }}>
            <Spin />
            <Text style={{ marginLeft: 10 }}>Processing audio...</Text>
          </div>
        )}
      </Space>
    </Modal>
  );
}
