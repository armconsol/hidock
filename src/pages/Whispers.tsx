import { useState, useEffect } from 'react';
import { Card, Space, Typography, Message, Modal } from '@arco-design/web-react';
import { WhisperList } from '../components/Whispers/WhisperList';
import { WhisperRecorder } from '../components/Whispers/WhisperRecorder';
import type { Whisper } from '../types/whispers';
import './Whispers.css';

const { Title } = Typography;

export function WhispersPage() {
  const [whispers, setWhispers] = useState<Whisper[]>([]);
  const [loading, setLoading] = useState(false);
  const [audioModalVisible, setAudioModalVisible] = useState(false);
  const [currentAudioUrl, setCurrentAudioUrl] = useState<string | null>(null);

  // Mock data for demonstration - in production, fetch from API
  useEffect(() => {
    // Simulate API call
    setLoading(true);
    setTimeout(() => {
      setWhispers([
        {
          id: '1',
          content: 'Remember to buy groceries: milk, eggs, bread, and coffee.',
          audioUrl: '/mock-audio-1.webm',
          duration: 15,
          createdAt: new Date(Date.now() - 1000 * 60 * 30), // 30 mins ago
          updatedAt: new Date(Date.now() - 1000 * 60 * 30),
        },
        {
          id: '2',
          content:
            'Meeting notes: Discussed Q4 goals with the team. Key priorities are improving user experience, launching new features, and increasing customer retention.',
          audioUrl: '/mock-audio-2.webm',
          duration: 45,
          createdAt: new Date(Date.now() - 1000 * 60 * 60 * 2), // 2 hours ago
          updatedAt: new Date(Date.now() - 1000 * 60 * 60 * 2),
        },
        {
          id: '3',
          content: 'Call mom on Sunday at 3 PM.',
          audioUrl: '/mock-audio-3.webm',
          duration: 8,
          createdAt: new Date(Date.now() - 1000 * 60 * 60 * 24), // 1 day ago
          updatedAt: new Date(Date.now() - 1000 * 60 * 60 * 24),
        },
      ]);
      setLoading(false);
    }, 500);
  }, []);

  const handleRecordingComplete = async (audioBlob: Blob, duration: number) => {
    try {
      // In production:
      // 1. Upload audio to backend
      // 2. Backend transcribes audio
      // 3. Create whisper note via API
      // 4. Add to list

      // Mock implementation
      const audioUrl = URL.createObjectURL(audioBlob);
      const newWhisper: Whisper = {
        id: Date.now().toString(),
        content: 'Transcription in progress...', // Would be transcribed by backend
        audioUrl,
        duration,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      setWhispers((prev) => [newWhisper, ...prev]);
      Message.success('Whisper recorded successfully');

      // Simulate transcription delay
      setTimeout(() => {
        setWhispers((prev) =>
          prev.map((w) =>
            w.id === newWhisper.id
              ? { ...w, content: 'This is a mock transcription of your audio recording.' }
              : w
          )
        );
      }, 2000);
    } catch (err) {
      Message.error('Failed to save recording');
      console.error('Recording save error:', err);
    }
  };

  const handleDelete = async (whisperId: string) => {
    Modal.confirm({
      title: 'Delete Whisper',
      content: 'Are you sure you want to delete this whisper? This action cannot be undone.',
      onOk: async () => {
        try {
          // In production: Call DELETE API
          // await api.delete(`/v1/note/delete`, { data: { id: whisperId } });

          setWhispers((prev) => prev.filter((w) => w.id !== whisperId));
          Message.success('Whisper deleted');
        } catch (err) {
          Message.error('Failed to delete whisper');
          console.error('Delete error:', err);
        }
      },
    });
  };

  const handleConvert = async (
    whisperId: string,
    targetType: 'note' | 'todo' | 'calendar'
  ) => {
    try {
      // In production: Call appropriate conversion API
      // - /v1/note/whisper/create/note
      // - /v1/note/whisper/add/todo
      // - /v1/note/whisper/extract/calendar

      const apiEndpoints = {
        note: '/v1/note/whisper/create/note',
        todo: '/v1/note/whisper/add/todo',
        calendar: '/v1/note/whisper/extract/calendar',
      };

      console.log(`Converting whisper ${whisperId} to ${targetType} via ${apiEndpoints[targetType]}`);

      Message.success(`Whisper converted to ${targetType}`);
    } catch (err) {
      Message.error(`Failed to convert to ${targetType}`);
      console.error('Conversion error:', err);
    }
  };

  const handlePlayAudio = (audioUrl: string) => {
    setCurrentAudioUrl(audioUrl);
    setAudioModalVisible(true);
  };

  return (
    <div className="whispers-page">
      <div className="whispers-header">
        <Title heading={3}>Whispers</Title>
      </div>

      <div className="whispers-content">
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          {/* Recorder Card */}
          <Card bordered={false}>
            <WhisperRecorder
              onRecordingComplete={handleRecordingComplete}
              maxDuration={120}
            />
          </Card>

          {/* Whispers List */}
          <Card bordered={false}>
            <WhisperList
              whispers={whispers}
              loading={loading}
              onDelete={handleDelete}
              onConvert={handleConvert}
              onPlayAudio={handlePlayAudio}
            />
          </Card>
        </Space>
      </div>

      {/* Audio Playback Modal */}
      <Modal
        title="Play Audio"
        visible={audioModalVisible}
        onCancel={() => setAudioModalVisible(false)}
        footer={null}
        style={{ width: 600 }}
      >
        {currentAudioUrl && (
          <audio controls style={{ width: '100%' }}>
            <source src={currentAudioUrl} type="audio/webm" />
            Your browser does not support the audio element.
          </audio>
        )}
      </Modal>
    </div>
  );
}
