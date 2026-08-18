import { useState, useEffect } from 'react';
import {
  Input,
  Button,
  Space,
  Typography,
  Tag,
  Dropdown,
  Menu,
  Message,
} from '@arco-design/web-react';
import {
  IconMore,
  IconDelete,
  IconArchive,
  IconPushpin,
  IconTag,
  IconSave,
} from '@arco-design/web-react/icon';
import { useNotesStore } from '../../store/notesStore';
import type { Note } from '../../types/notes';
import './NoteDetail.css';

const { TextArea } = Input;
const { Title, Text } = Typography;

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
      Message.success('Note saved');
    }
  };

  const handleDelete = () => {
    if (selectedNoteId) {
      deleteNote(selectedNoteId);
      Message.success('Note deleted');
    }
  };

  const handlePin = () => {
    if (selectedNoteId) {
      togglePinNote(selectedNoteId);
      Message.success(selectedNote?.isPinned ? 'Note unpinned' : 'Note pinned');
    }
  };

  const handleArchive = () => {
    if (selectedNoteId) {
      toggleArchiveNote(selectedNoteId);
      Message.success(
        selectedNote?.isArchived ? 'Note unarchived' : 'Note archived'
      );
    }
  };

  const dropdownMenu = (
    <Menu>
      <Menu.Item key="pin" onClick={handlePin}>
        <Space>
          <IconPushpin />
          {selectedNote?.isPinned ? 'Unpin' : 'Pin'}
        </Space>
      </Menu.Item>
      <Menu.Item key="archive" onClick={handleArchive}>
        <Space>
          <IconArchive />
          {selectedNote?.isArchived ? 'Unarchive' : 'Archive'}
        </Space>
      </Menu.Item>
      <Menu.Item key="delete" onClick={handleDelete}>
        <Space>
          <IconDelete />
          Delete
        </Space>
      </Menu.Item>
    </Menu>
  );

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
            onChange={setTitle}
            placeholder="Note title"
            style={{ fontSize: 20, fontWeight: 600 }}
            bordered={false}
          />
          <Space size={12} wrap>
            <Text type="secondary" style={{ fontSize: 12 }}>
              Modified {formatDate(selectedNote.updatedAt)}
            </Text>
            {selectedNote.tags && selectedNote.tags.length > 0 && (
              <>
                {selectedNote.tags.map((tag) => (
                  <Tag key={tag} size="small" color="arcoblue">
                    {tag}
                  </Tag>
                ))}
              </>
            )}
          </Space>
        </Space>
        <Space size={8}>
          {hasChanges && (
            <Button type="primary" icon={<IconSave />} onClick={handleSave}>
              Save
            </Button>
          )}
          <Dropdown droplist={dropdownMenu} position="br">
            <Button icon={<IconMore />} />
          </Dropdown>
        </Space>
      </div>

      <div className="note-detail-content">
        <TextArea
          value={content}
          onChange={setContent}
          placeholder="Start writing..."
          autoSize={{ minRows: 10 }}
          style={{ fontSize: 14, lineHeight: 1.6 }}
          bordered={false}
        />
      </div>
    </div>
  );
}
