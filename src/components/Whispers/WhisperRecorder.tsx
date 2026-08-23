import { Button, Space, Typography, Modal, message } from 'antd';
import { StopOutlined, AudioOutlined } from '@ant-design/icons';
import { useState, useRef, useEffect } from 'react';
import './WhisperRecorder.css';

const { Text } = Typography;

interface WhisperRecorderProps {
  onRecordingComplete?: (audioBlob: Blob, duration: number) => void;
  maxDuration?: number; // Maximum recording duration in seconds
}

export function WhisperRecorder({
  onRecordingComplete,
  maxDuration = 120, // Default 2 minutes
}: WhisperRecorderProps) {
  const [isRecording, setIsRecording] = useState(false);
  const [isPressing, setIsPressing] = useState(false);
  const [recordingDuration, setRecordingDuration] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioChunksRef = useRef<Blob[]>([]);
  const timerRef = useRef<number | null>(null);
  const startTimeRef = useRef<number>(0);

  useEffect(() => {
    return () => {
      // Cleanup on unmount
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
      if (mediaRecorderRef.current && isRecording) {
        mediaRecorderRef.current.stop();
      }
    };
  }, [isRecording]);

  const startRecording = async () => {
    try {
      setError(null);
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });

      const mediaRecorder = new MediaRecorder(stream, {
        mimeType: 'audio/webm',
      });

      audioChunksRef.current = [];

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          audioChunksRef.current.push(event.data);
        }
      };

      mediaRecorder.onstop = () => {
        const audioBlob = new Blob(audioChunksRef.current, { type: 'audio/webm' });
        const duration = Math.floor((Date.now() - startTimeRef.current) / 1000);

        // Stop all audio tracks
        stream.getTracks().forEach((track) => track.stop());

        if (duration > 0) {
          onRecordingComplete?.(audioBlob, duration);
          message.success('Recording completed');
        }

        setIsRecording(false);
        setRecordingDuration(0);
        if (timerRef.current) {
          clearInterval(timerRef.current);
        }
      };

      mediaRecorder.start(100); // Collect data every 100ms
      mediaRecorderRef.current = mediaRecorder;
      setIsRecording(true);
      startTimeRef.current = Date.now();

      // Start timer
      timerRef.current = setInterval(() => {
        const elapsed = Math.floor((Date.now() - startTimeRef.current) / 1000);
        setRecordingDuration(elapsed);

        // Auto-stop at max duration
        if (elapsed >= maxDuration) {
          stopRecording();
          message.warning(`Recording stopped: ${maxDuration}s limit reached`);
        }
      }, 100);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : 'Failed to access microphone';
      setError(errorMessage);
      message.error(errorMessage);
      console.error('Recording error:', err);
    }
  };

  const stopRecording = () => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop();
    }
  };

  const handleMouseDown = () => {
    setIsPressing(true);
    startRecording();
  };

  const handleMouseUp = () => {
    setIsPressing(false);
    if (isRecording) {
      stopRecording();
    }
  };

  const handleMouseLeave = () => {
    if (isPressing && isRecording) {
      // Show confirmation modal if user drags away while recording
      Modal.confirm({
        title: 'Continue Recording?',
        content: 'You dragged away from the button. Stop recording?',
        onOk: stopRecording,
      });
    }
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="whisper-recorder" data-testid="whisper-recorder">
      <Space direction="vertical" size="medium" style={{ width: '100%' }}>
        {/* Recording button */}
        <div className="whisper-recorder-button-container">
          <Button
            type="primary"
            size="large"
            shape="circle"
            icon={isRecording ? <StopOutlined /> : <AudioOutlined />}
            className={`whisper-record-button ${isRecording ? 'recording' : ''}`}
            onMouseDown={handleMouseDown}
            onMouseUp={handleMouseUp}
            onMouseLeave={handleMouseLeave}
            onTouchStart={handleMouseDown}
            onTouchEnd={handleMouseUp}
            data-testid="record-button"
            style={{
              width: 80,
              height: 80,
              fontSize: 32,
            }}
          />
        </div>

        {/* Instructions */}
        <div className="whisper-recorder-instructions">
          {isRecording ? (
            <Space direction="vertical" align="center" size="small">
              <Text type="danger" style={{ fontSize: 16, fontWeight: 500 }}>
                Recording... {formatDuration(recordingDuration)}
              </Text>
              <Text type="secondary" style={{ fontSize: 12 }}>
                Release to stop
              </Text>
            </Space>
          ) : (
            <Space direction="vertical" align="center" size="small">
              <Text style={{ fontSize: 14 }}>Press and hold to record</Text>
              <Text type="secondary" style={{ fontSize: 12 }}>
                Max duration: {formatDuration(maxDuration)}
              </Text>
            </Space>
          )}
        </div>

        {/* Error display */}
        {error && (
          <div className="whisper-recorder-error" data-testid="recorder-error">
            <Text type="danger">{error}</Text>
          </div>
        )}
      </Space>
    </div>
  );
}
