import { NotesList } from './NotesList';
import { NoteDetail } from './NoteDetail';
import './NotesView.css';

export function NotesView() {
  return (
    <div className="notes-view">
      <div className="notes-view-sidebar">
        <NotesList />
      </div>
      <div className="notes-view-content">
        <NoteDetail />
      </div>
    </div>
  );
}
