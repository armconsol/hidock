import { Button, Slider, Space, Spin, Typography } from '@arco-design/web-react';
import {
  IconPause,
  IconPlayArrow,
  IconDownload,
  IconSound,
} from '@arco-design/web-react/icon';
import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './AudioPlayer.css';

const { Text } = Typography;

interface AudioPlayerProps {
  noteId: string;
  audioUrl: string;
  className?: string;
}

export function AudioPlayer({ noteId, audioUrl, className = '' }: AudioPlayerProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [volume, setVolume] = useState(1);
  const [playbackRate, setPlaybackRate] = useState(1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [audioBlob, setAudioBlob] = useState<string | null>(null);

  // Fetch audio from Tauri backend
  useEffect(() => {
    const fetchAudio = async () => {
      try {
        setLoading(true);
        setError(null);

        // Invoke Tauri command to get audio
        const audioData = await invoke<number[]>('get_audio', {
          noteId,
          audioUrl,
        });

        // Convert array to Uint8Array and create blob
        const uint8Array = new Uint8Array(audioData);
        const blob = new Blob([uint8Array], { type: 'audio/mpeg' });
        const blobUrl = URL.createObjectURL(blob);

        setAudioBlob(blobUrl);
        setLoading(false);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load audio');
        setLoading(false);
        console.error('Error loading audio:', err);
      }
    };

    fetchAudio();

    // Cleanup blob URL on unmount
    return () => {
      if (audioBlob) {
        URL.revokeObjectURL(audioBlob);
      }
    };
  }, [noteId, audioUrl]);

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

  // Update volume
  useEffect(() => {
    if (audioRef.current) {
      audioRef.current.volume = volume;
    }
  }, [volume]);

  // Update playback rate
  useEffect(() => {
    if (audioRef.current) {
      audioRef.current.playbackRate = playbackRate;
    }
  }, [playbackRate]);

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

  const handleSeek = (value: number) => {
    const audio = audioRef.current;
    if (!audio) return;

    audio.currentTime = value;
    setCurrentTime(value);
  };

  const handleVolumeChange = (value: number) => {
    setVolume(value / 100);
  };

  const handleSpeedChange = (speed: number) => {
    setPlaybackRate(speed);
  };

  const handleDownload = () => {
    if (!audioBlob) return;

    const link = document.createElement('a');
    link.href = audioBlob;
    link.download = `audio-${noteId}.mp3`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const formatTime = (seconds: number): string => {
    if (isNaN(seconds) || !isFinite(seconds)) return '0:00';

    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  if (loading) {
    return (
      <div className={`audio-player ${className}`} data-testid="audio-player-loading">
        <Spin />
        <Text style={{ marginLeft: 12 }}>Loading audio...</Text>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`audio-player ${className}`} data-testid="audio-player-error">
        <Text type="danger">{error}</Text>
      </div>
    );
  }

  return (
    <div className={`audio-player ${className}`} data-testid="audio-player">
      <audio ref={audioRef} src={audioBlob || undefined} />

      <div className="audio-player-controls">
        <Space size="medium">
          {/* Play/Pause Button */}
          <Button
            type="primary"
            shape="circle"
            icon={isPlaying ? <IconPause /> : <IconPlayArrow />}
            onClick={togglePlayPause}
            data-testid="play-pause-button"
          />

          {/* Progress Bar */}
          <div className="audio-player-progress">
            <Slider
              value={currentTime}
              max={duration}
              onChange={handleSeek}
              showTooltip={false}
              style={{ width: 300, margin: 0 }}
              data-testid="progress-slider"
            />
            <div className="audio-player-time">
              <Text type="secondary">
                {formatTime(currentTime)} / {formatTime(duration)}
              </Text>
            </div>
          </div>

          {/* Volume Control */}
          <div className="audio-player-volume">
            <IconSound style={{ marginRight: 8 }} />
            <Slider
              value={volume * 100}
              max={100}
              onChange={handleVolumeChange}
              showTooltip={false}
              style={{ width: 100, margin: 0 }}
              data-testid="volume-slider"
            />
          </div>

          {/* Speed Control */}
          <Space size="small">
            {[0.5, 1, 1.5, 2].map((speed) => (
              <Button
                key={speed}
                size="small"
                type={playbackRate === speed ? 'primary' : 'default'}
                onClick={() => handleSpeedChange(speed)}
                data-testid={`speed-button-${speed}`}
              >
                {speed}x
              </Button>
            ))}
          </Space>

          {/* Download Button */}
          <Button
            icon={<IconDownload />}
            onClick={handleDownload}
            data-testid="download-button"
          >
            Download
          </Button>
        </Space>
      </div>
    </div>
  );
}
