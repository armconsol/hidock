import { useState, useEffect } from 'react';
import {
  Input,
  Button,
  Space,
  Typography,
  Tag,
  Dropdown,
  message,
} from 'antd';
import {
  MoreOutlined,
  DeleteOutlined,
  InboxOutlined,
  PushpinOutlined,
  SaveOutlined,
} from '@ant-design/icons';
import { useNotesStore } from '../../store/notesStore';
import './NoteDetail.css';

const { TextArea } = Input;
const { Text } = Typography;

export function NoteDetail() {
  const {
    notes,
    selectedNoteId,
    updateNote,
    deleteNote,
    togglePinNote,
    toggleArchiveNote,
  } = useNotesStore();

  const selectedNote = notes.find((note) => note.id === selectedNoteId);

  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    if (selectedNote) {
      setTitle(selectedNote.title);
      setContent(selectedNote.content);
      setHasChanges(false);
    }
  }, [selectedNote]);

  useEffect(() => {
    if (selectedNote) {
      const changed =
        title !== selectedNote.title || content !== selectedNote.content;
      setHasChanges(changed);
    }
  }, [title, content, selectedNote]);

  const handleSave = () => {
    if (selectedNoteId && hasChanges) {
      updateNote(selectedNoteId, { title, content });
      setHasChanges(false);
      message.success('Note saved');
    }
  };

  const handleDelete = () => {
    if (selectedNoteId) {
      deleteNote(selectedNoteId);
      message.success('Note deleted');
    }
  };

  const handlePin = () => {
    if (selectedNoteId) {
      togglePinNote(selectedNoteId);
      message.success(selectedNote?.isPinned ? 'Note unpinned' : 'Note pinned');
    }
  };

  const handleArchive = () => {
    if (selectedNoteId) {
      toggleArchiveNote(selectedNoteId);
      message.success(
        selectedNote?.isArchived ? 'Note unarchived' : 'Note archived'
      );
    }
  };

  const dropdownMenu = {
    items: [
      {
        key: 'pin',
        label: (
          <Space>
            <PushpinOutlined />
            {selectedNote?.isPinned ? 'Unpin' : 'Pin'}
          </Space>
        ),
        onClick: handlePin,
      },
      {
        key: 'archive',
        label: (
          <Space>
            <InboxOutlined />
            {selectedNote?.isArchived ? 'Unarchive' : 'Archive'}
          </Space>
        ),
        onClick: handleArchive,
      },
      {
        key: 'delete',
        label: (
          <Space>
            <DeleteOutlined />
            Delete
          </Space>
        ),
        onClick: handleDelete,
      },
    ],
  };

  if (!selectedNote) {
    return (
      <div className="note-detail-empty">
        <Text type="secondary">Select a note to view or create a new one</Text>
      </div>
    );
  }

  const formatDate = (date: Date): string => {
    return new Date(date).toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  };

  return (
    <div className="note-detail">
      <div className="note-detail-header">
        <Space direction="vertical" size={8} style={{ flex: 1 }}>
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Note title"
            style={{ fontSize: 20, fontWeight: 600 }}
          />
          <Space size={12} wrap>
            <Text type="secondary" style={{ fontSize: 12 }}>
              Modified {formatDate(selectedNote.updatedAt)}
            </Text>
            {selectedNote.tags && selectedNote.tags.length > 0 && (
              <>
                {selectedNote.tags.map((tag) => (
                  <Tag key={tag} color="blue">
                    {tag}
                  </Tag>
                ))}
              </>
            )}
          </Space>
        </Space>
        <Space size={8}>
          {hasChanges && (
            <Button type="primary" icon={<SaveOutlined />} onClick={handleSave}>
              Save
            </Button>
          )}
          <Dropdown menu={dropdownMenu} placement="bottomRight">
            <Button icon={<MoreOutlined />} />
          </Dropdown>
        </Space>
      </div>

      <div className="note-detail-content">
        <TextArea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="Start writing..."
          autoSize={{ minRows: 10 }}
          style={{ fontSize: 14, lineHeight: 1.6 }}
        />
      </div>
    </div>
  );
}
