import { Button, Space, Spin, Typography, Message } from '@arco-design/web-react';
import {
  IconPlayArrow,
  IconPause,
  IconCopy,
  IconSwap,
  IconSave,
} from '@arco-design/web-react/icon';
import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { MergeDialog } from './MergeDialog';
import { ReplaceDialog } from './ReplaceDialog';
import { SaveAsNewDialog } from './SaveAsNewDialog';
import './AudioEditor.css';

const { Text } = Typography;

interface AudioEditorProps {
  noteId: string;
  audioUrl: string;
  className?: string;
  onOperationComplete?: (outputPath: string) => void;
}

interface TimelineMarker {
  id: string;
  time: number;
  type: 'merge' | 'replace' | 'custom';
}

export function AudioEditor({
  noteId,
  audioUrl,
  className = '',
  onOperationComplete,
}: AudioEditorProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const timelineRef = useRef<HTMLDivElement>(null);

  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [audioBlob, setAudioBlob] = useState<string | null>(null);
  const [audioData, setAudioData] = useState<Uint8Array | null>(null);

  const [markers, setMarkers] = useState<TimelineMarker[]>([]);
  const [selectedRange, setSelectedRange] = useState<{ start: number; end: number } | null>(
    null
  );
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState<number | null>(null);

  // Dialog states
  const [mergeDialogVisible, setMergeDialogVisible] = useState(false);
  const [replaceDialogVisible, setReplaceDialogVisible] = useState(false);
  const [saveAsNewDialogVisible, setSaveAsNewDialogVisible] = useState(false);

  // Fetch audio from Tauri backend
  useEffect(() => {
    const fetchAudio = async () => {
      try {
        setLoading(true);
        setError(null);

        const audioDataArray = await invoke<number[]>('get_audio', {
          noteId,
          audioUrl,
        });

        const uint8Array = new Uint8Array(audioDataArray);
        const blob = new Blob([uint8Array], { type: 'audio/mpeg' });
        const blobUrl = URL.createObjectURL(blob);

        setAudioData(uint8Array);
        setAudioBlob(blobUrl);
        setLoading(false);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load audio');
        setLoading(false);
        console.error('Error loading audio:', err);
      }
    };

    fetchAudio();

    return () => {
      if (audioBlob) {
        URL.revokeObjectURL(audioBlob);
      }
    };
  }, [noteId, audioUrl]);

  // Draw waveform when audio is loaded
  useEffect(() => {
    if (audioData && canvasRef.current) {
      drawWaveform(audioData);
    }
  }, [audioData]);

  // Update current time
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const updateTime = () => setCurrentTime(audio.currentTime);
    const updateDuration = () => setDuration(audio.duration);
    const handleEnded = () => setIsPlaying(false);

    audio.addEventListener('timeupdate', updateTime);
    audio.addEventListener('loadedmetadata', updateDuration);
    audio.addEventListener('ended', handleEnded);

    return () => {
      audio.removeEventListener('timeupdate', updateTime);
      audio.removeEventListener('loadedmetadata', updateDuration);
      audio.removeEventListener('ended', handleEnded);
    };
  }, [audioBlob]);

  const drawWaveform = (data: Uint8Array) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;

    // Clear canvas
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, width, height);

    // Draw waveform
    const samples = Math.min(data.length, width);
    const step = Math.floor(data.length / samples);
    const amp = height / 2;

    ctx.beginPath();
    ctx.strokeStyle = '#3491fa';
    ctx.lineWidth = 1;

    for (let i = 0; i < samples; i++) {
      const x = (i / samples) * width;
      const y = amp - (data[i * step] / 255) * amp;

      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }

    ctx.stroke();

    // Draw selected range if exists
    if (selectedRange) {
      const startX = (selectedRange.start / duration) * width;
      const endX = (selectedRange.end / duration) * width;

      ctx.fillStyle = 'rgba(52, 145, 250, 0.2)';
      ctx.fillRect(startX, 0, endX - startX, height);
    }
  };

  const togglePlayPause = () => {
    const audio = audioRef.current;
    if (!audio) return;

    if (isPlaying) {
      audio.pause();
    } else {
      audio.play();
    }
    setIsPlaying(!isPlaying);
  };

  const handleTimelineClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!timelineRef.current || !audioRef.current) return;

    const rect = timelineRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percentage = x / rect.width;
    const newTime = percentage * duration;

    audioRef.current.currentTime = newTime;
    setCurrentTime(newTime);
  };

  const handleTimelineMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    setIsDragging(true);
    const rect = timelineRef.current?.getBoundingClientRect();
    if (rect) {
      const x = e.clientX - rect.left;
      const percentage = x / rect.width;
      setDragStart(percentage * duration);
    }
  };

  const handleTimelineMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!isDragging || dragStart === null || !timelineRef.current) return;

    const rect = timelineRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percentage = x / rect.width;
    const currentDragTime = percentage * duration;

    setSelectedRange({
      start: Math.min(dragStart, currentDragTime),
      end: Math.max(dragStart, currentDragTime),
    });

    // Redraw waveform with selection
    if (audioData) {
      drawWaveform(audioData);
    }
  };

  const handleTimelineMouseUp = () => {
    setIsDragging(false);
    setDragStart(null);
  };

  const addMarker = () => {
    const newMarker: TimelineMarker = {
      id: `marker-${Date.now()}`,
      time: currentTime,
      type: 'custom',
    };
    setMarkers([...markers, newMarker]);
  };

  const formatTime = (seconds: number): string => {
    if (isNaN(seconds) || !isFinite(seconds)) return '0:00';

    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const handleMergeComplete = (outputPath: string) => {
    Message.success('Audio files merged successfully');
    if (onOperationComplete) {
      onOperationComplete(outputPath);
    }
  };

  const handleReplaceComplete = (outputPath: string) => {
    Message.success('Audio segment replaced successfully');
    if (onOperationComplete) {
      onOperationComplete(outputPath);
    }
  };

  const handleSaveAsNewComplete = (outputPath: string) => {
    Message.success('Audio saved as new file');
    if (onOperationComplete) {
      onOperationComplete(outputPath);
    }
  };

  if (loading) {
    return (
      <div className={`audio-editor ${className}`}>
        <Space>
          <Spin />
          <Text>Loading audio...</Text>
        </Space>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`audio-editor ${className}`}>
        <Text type="error">{error}</Text>
      </div>
    );
  }

  return (
    <div className={`audio-editor ${className}`} data-testid="audio-editor">
      <audio ref={audioRef} src={audioBlob || undefined} />

      {/* Waveform Canvas */}
      <canvas
        ref={canvasRef}
        data-testid="waveform-canvas"
        width={800}
        height={120}
        style={{ width: '100%', maxWidth: '800px', height: '120px', cursor: 'crosshair' }}
      />

      {/* Timeline */}
      <div
        ref={timelineRef}
        data-testid="timeline"
        className="audio-editor-timeline"
        onClick={handleTimelineClick}
        onMouseDown={handleTimelineMouseDown}
        onMouseMove={handleTimelineMouseMove}
        onMouseUp={handleTimelineMouseUp}
        onMouseLeave={handleTimelineMouseUp}
        style={{
          position: 'relative',
          width: '100%',
          height: '40px',
          backgroundColor: '#2a2a2a',
          marginTop: '10px',
          cursor: isDragging ? 'col-resize' : 'pointer',
        }}
      >
        {/* Playback Indicator */}
        <div
          data-testid="playback-indicator"
          style={{
            position: 'absolute',
            left: `${(currentTime / duration) * 100}%`,
            top: 0,
            width: '2px',
            height: '100%',
            backgroundColor: '#3491fa',
          }}
        />

        {/* Markers */}
        {markers.map((marker) => (
          <div
            key={marker.id}
            data-testid="timeline-marker"
            style={{
              position: 'absolute',
              left: `${(marker.time / duration) * 100}%`,
              top: 0,
              width: '2px',
              height: '100%',
              backgroundColor: marker.type === 'merge' ? '#00d68f' : '#ff6b72',
            }}
          />
        ))}

        {/* Time labels */}
        <div style={{ display: 'flex', justifyContent: 'space-between', padding: '5px' }}>
          <Text type="secondary">{formatTime(currentTime)}</Text>
          <Text type="secondary">{formatTime(duration)}</Text>
        </div>
      </div>

      {/* Controls */}
      <div className="audio-editor-controls" style={{ marginTop: '20px' }}>
        <Space size="medium">
          <Button
            type="primary"
            shape="circle"
            icon={isPlaying ? <IconPause /> : <IconPlayArrow />}
            onClick={togglePlayPause}
            data-testid="play-pause-button"
          />

          <Button
            icon={<IconCopy />}
            onClick={addMarker}
            data-testid="add-marker-button"
          >
            Add Marker
          </Button>

          <Button
            icon={<IconCopy />}
            onClick={() => setMergeDialogVisible(true)}
            data-testid="merge-button"
          >
            Merge Audio
          </Button>

          <Button
            icon={<IconSwap />}
            onClick={() => setReplaceDialogVisible(true)}
            data-testid="replace-button"
          >
            Replace Segment
          </Button>

          <Button
            icon={<IconSave />}
            onClick={() => setSaveAsNewDialogVisible(true)}
            data-testid="save-as-new-button"
            disabled={!audioData}
          >
            Save As New
          </Button>
        </Space>
      </div>

      {/* Dialogs */}
      <MergeDialog
        visible={mergeDialogVisible}
        onClose={() => setMergeDialogVisible(false)}
        onComplete={handleMergeComplete}
      />

      <ReplaceDialog
        visible={replaceDialogVisible}
        originalFilePath={audioUrl}
        selectedRange={selectedRange}
        onClose={() => setReplaceDialogVisible(false)}
        onComplete={handleReplaceComplete}
      />

      {audioData && (
        <SaveAsNewDialog
          visible={saveAsNewDialogVisible}
          audioData={audioData}
          onClose={() => setSaveAsNewDialogVisible(false)}
          onComplete={handleSaveAsNewComplete}
        />
      )}
    </div>
  );
}
