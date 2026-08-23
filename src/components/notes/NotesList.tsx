import { Space, Input, Button, Select, Empty } from 'antd';
import {
  SearchOutlined,
  PlusOutlined,
  FilterOutlined,
  SortAscendingOutlined,
} from '@ant-design/icons';
import { useNotesStore } from '../../store/notesStore';
import { NoteCard } from './NoteCard';
import type { NoteSortBy, SortOrder } from '../../types/notes';
import './NotesList.css';


export function NotesList() {
  const {
    selectedNoteId,
    filter,
    sortBy,
    sortOrder,
    selectNote,
    setFilter,
    setSorting,
    addNote,
    getFilteredNotes,
  } = useNotesStore();

  const notes = getFilteredNotes();

  const handleSearch = (value: string) => {
    setFilter({ ...filter, searchQuery: value });
  };

  const handleSortChange = (value: string) => {
    const [newSortBy, newSortOrder] = value.split('-') as [NoteSortBy, SortOrder];
    setSorting(newSortBy, newSortOrder);
  };

  const handleCreateNote = () => {
    addNote({
      title: 'Untitled Note',
      content: '',
    });
  };

  const sortOptions = [
    { label: 'Last Modified (Newest)', value: 'updatedAt-desc' },
    { label: 'Last Modified (Oldest)', value: 'updatedAt-asc' },
    { label: 'Date Created (Newest)', value: 'createdAt-desc' },
    { label: 'Date Created (Oldest)', value: 'createdAt-asc' },
    { label: 'Title (A-Z)', value: 'title-asc' },
    { label: 'Title (Z-A)', value: 'title-desc' },
  ];

  return (
    <div className="notes-list">
      <div className="notes-list-header">
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          <div className="notes-list-actions">
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={handleCreateNote}
              style={{ width: '100%' }}
            >
              New Note
            </Button>
          </div>
          <Input
            allowClear
            placeholder="Search notes..."
            prefix={<SearchOutlined />}
            value={filter.searchQuery || ''}
            onChange={(e) => handleSearch(e.target.value)}
          />
          <Space size={8} style={{ width: '100%' }}>
            <Select
              placeholder="Sort by"
              value={`${sortBy}-${sortOrder}`}
              onChange={handleSortChange}
              style={{ flex: 1 }}
              suffixIcon={<SortAscendingOutlined />}
              options={sortOptions}
            />
            <Button icon={<FilterOutlined />} />
          </Space>
        </Space>
      </div>

      <div className="notes-list-content">
        {notes.length === 0 ? (
          <Empty
            description={
              filter.searchQuery
                ? 'No notes found'
                : 'No notes yet. Create your first note!'
            }
          />
        ) : (
          <Space direction="vertical" size={8} style={{ width: '100%' }}>
            {notes.map((note) => (
              <NoteCard
                key={note.id}
                note={note}
                isSelected={note.id === selectedNoteId}
                onClick={() => selectNote(note.id)}
              />
            ))}
          </Space>
        )}
      </div>
    </div>
  );
}
