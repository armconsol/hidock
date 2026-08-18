// Widget types for customizable dashboard

export type WidgetType =
  | 'recent-notes'
  | 'whispers'
  | 'calendar'
  | 'events'
  | 'todos'
  | 'quote';

export interface WidgetPosition {
  column: number; // 0, 1, or 2 (left, center, right)
  row: number;
}

export interface WidgetSize {
  width: number; // Column span
  height: number; // Pixels or grid units
}

export interface Widget {
  id: string;
  type: WidgetType;
  position: WidgetPosition;
  size: WidgetSize;
  visible: boolean;
}

export interface DashboardLayout {
  widgets: Widget[];
  columns: number; // Typically 3
}

export const DEFAULT_LAYOUT: DashboardLayout = {
  columns: 3,
  widgets: [
    // Left column
    { id: 'recent-notes', type: 'recent-notes', position: { column: 0, row: 0 }, size: { width: 1, height: 500 }, visible: true },
    { id: 'whispers', type: 'whispers', position: { column: 0, row: 1 }, size: { width: 1, height: 400 }, visible: true },

    // Center column
    { id: 'calendar', type: 'calendar', position: { column: 1, row: 0 }, size: { width: 1, height: 450 }, visible: true },
    { id: 'events', type: 'events', position: { column: 1, row: 1 }, size: { width: 1, height: 450 }, visible: true },

    // Right column
    { id: 'todos', type: 'todos', position: { column: 2, row: 0 }, size: { width: 1, height: 600 }, visible: true },
    { id: 'quote', type: 'quote', position: { column: 2, row: 1 }, size: { width: 1, height: 200 }, visible: true },
  ],
};
