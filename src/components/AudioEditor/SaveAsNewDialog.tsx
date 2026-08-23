import { Modal, Button, Typography, Space, Select, Slider, Spin } from 'antd';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

const { Text, Title } = Typography;

interface SaveAsNewDialogProps {
  visible: boolean;
  audioData: Uint8Array;
  onClose: () => void;
  onComplete: (outputPath: string) => void;
}

interface AudioOperationResult {
  success: boolean;
  output_path?: string;
  error?: string;
}

type AudioFormat = 'mp3' | 'm4a' | 'wav' | 'aac' | 'ogg' | 'flac';

const FORMATS: { value: AudioFormat; label: string; lossy: boolean }[] = [
  { value: 'mp3', label: 'MP3', lossy: true },
  { value: 'm4a', label: 'M4A', lossy: true },
  { value: 'aac', label: 'AAC', lossy: true },
  { value: 'ogg', label: 'OGG', lossy: true },
  { value: 'wav', label: 'WAV', lossy: false },
  { value: 'flac', label: 'FLAC', lossy: false },
];

const BITRATES = ['128k', '192k', '256k', '320k'];

export function SaveAsNewDialog({
  visible,
  audioData,
  onClose,
  onComplete,
}: SaveAsNewDialogProps) {
  const [format, setFormat] = useState<AudioFormat>('mp3');
  const [quality, setQuality] = useState<number>(192); // Default bitrate
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectedFormat = FORMATS.find((f) => f.value === format);
  const isLossy = selectedFormat?.lossy || false;

  const handleSave = async () => {
    setIsProcessing(true);
    setError(null);

    try {
      const result = await invoke<AudioOperationResult>('save_audio_as_new', {
        audio_data: Array.from(audioData),
        format: format,
      });

      if (result.success && result.output_path) {
        onComplete(result.output_path);
        handleClose();
      } else {
        setError(result.error || 'Failed to save audio file');
      }
    } catch (err) {
      console.error('Save failed:', err);
      setError(`Failed to save audio: ${err}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleClose = () => {
    setFormat('mp3');
    setQuality(192);
    setError(null);
    setIsProcessing(false);
    onClose();
  };

  const getEstimatedSize = (): string => {
    // Rough estimation based on format and quality
    // Assuming 1 minute of audio at 192kbps = ~1.44MB
    const durationMinutes = audioData.length / (44100 * 2 * 60); // Rough estimate

    if (isLossy) {
      const sizeKB = (quality * durationMinutes * 60) / 8;
      if (sizeKB > 1024) {
        return `~${(sizeKB / 1024).toFixed(1)} MB`;
      }
      return `~${sizeKB.toFixed(0)} KB`;
    } else {
      // Lossless formats are much larger
      const sizeKB = audioData.length / 1024;
      if (sizeKB > 1024) {
        return `~${(sizeKB / 1024).toFixed(1)} MB`;
      }
      return `~${sizeKB.toFixed(0)} KB`;
    }
  };

  const getQualityLabel = (value: number): string => {
    if (value < 128) return 'Low';
    if (value < 192) return 'Medium';
    if (value < 256) return 'High';
    return 'Very High';
  };

  return (
    <Modal
      title={<Title level={5}>Save As New</Title>}
      open={visible}
      onCancel={handleClose}
      footer={
        <Space>
          <Button onClick={handleClose} disabled={isProcessing}>
            Cancel
          </Button>
          <Button
            type="primary"
            onClick={handleSave}
            disabled={isProcessing}
            loading={isProcessing}
          >
            {isProcessing ? 'Saving...' : 'Save'}
          </Button>
        </Space>
      }
      width={600}
    >
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        {/* Format Selection */}
        <div>
          <Text strong style={{ marginBottom: 10, display: 'block' }}>
            Format
          </Text>
          <Select
            placeholder="Select audio format"
            value={format}
            onChange={(value) => setFormat(value as AudioFormat)}
            style={{ width: '100%' }}
            disabled={isProcessing}
            aria-label="Format"
            options={FORMATS.map((fmt) => ({
              value: fmt.value,
              label: `${fmt.label} ${fmt.lossy ? '(Lossy)' : '(Lossless)'}`,
            }))}
          />
        </div>

        {/* Quality Settings - Only for lossy formats */}
        {isLossy && (
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 10 }}>
              <Text strong>Quality</Text>
              <Text type="secondary">{getQualityLabel(quality)}</Text>
            </div>
            <Slider
              value={quality}
              onChange={(value) => setQuality(value as number)}
              min={64}
              max={320}
              step={32}
              disabled={isProcessing}
              marks={{
                64: '64k',
                128: '128k',
                192: '192k',
                256: '256k',
                320: '320k',
              }}
              style={{ width: '100%' }}
            />
          </div>
        )}

        {/* Bitrate Selection - Alternative to slider */}
        {isLossy && (
          <div>
            <Text strong style={{ marginBottom: 10, display: 'block' }}>
              Bitrate
            </Text>
            <Space>
              {BITRATES.map((bitrate) => {
                const bitrateValue = parseInt(bitrate);
                return (
                  <Button
                    key={bitrate}
                    type={quality === bitrateValue ? 'primary' : 'default'}
                    onClick={() => setQuality(bitrateValue)}
                    disabled={isProcessing}
                    size="small"
                  >
                    {bitrate}
                  </Button>
                );
              })}
            </Space>
          </div>
        )}

        {/* Estimated File Size */}
        <div
          style={{
            padding: '15px',
            backgroundColor: '#e8f3ff',
            borderRadius: '4px',
          }}
        >
          <Space direction="vertical" size="small">
            <div>
              <Text type="secondary">Format: </Text>
              <Text strong>{selectedFormat?.label}</Text>
            </div>
            {isLossy && (
              <div>
                <Text type="secondary">Bitrate: </Text>
                <Text strong>{quality}k</Text>
              </div>
            )}
            <div>
              <Text type="secondary">Estimated size: </Text>
              <Text strong>{getEstimatedSize()}</Text>
            </div>
          </Space>
        </div>

        {/* Format Information */}
        <div
          style={{
            padding: '10px',
            backgroundColor: '#f7f8fa',
            borderRadius: '4px',
          }}
        >
          <Text type="secondary" style={{ fontSize: 12 }}>
            {isLossy ? (
              <>
                <strong>Lossy format:</strong> Smaller file size with some quality loss. Good
                for sharing and storage.
              </>
            ) : (
              <>
                <strong>Lossless format:</strong> Original quality preserved. Larger file
                size.
              </>
            )}
          </Text>
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
            <Text style={{ marginLeft: 10 }}>Saving audio file...</Text>
          </div>
        )}
      </Space>
    </Modal>
  );
}
