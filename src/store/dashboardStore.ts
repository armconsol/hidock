import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { DashboardLayout, DEFAULT_LAYOUT } from '../types/widgets';

interface DashboardState {
  layout: DashboardLayout;

  // Actions
  updateWidgetPosition: (widgetId: string, column: number, row: number) => void;
  updateWidgetSize: (widgetId: string, width: number, height: number) => void;
  toggleWidgetVisibility: (widgetId: string) => void;
  resetLayout: () => void;
}

export const useDashboardStore = create<DashboardState>()(
  persist(
    (set) => ({
      layout: DEFAULT_LAYOUT,

      updateWidgetPosition: (widgetId, column, row) =>
        set((state) => ({
          layout: {
            ...state.layout,
            widgets: state.layout.widgets.map((widget) =>
              widget.id === widgetId
                ? { ...widget, position: { column, row } }
                : widget
            ),
          },
        })),

      updateWidgetSize: (widgetId, width, height) =>
        set((state) => ({
          layout: {
            ...state.layout,
            widgets: state.layout.widgets.map((widget) =>
              widget.id === widgetId
                ? { ...widget, size: { width, height } }
                : widget
            ),
          },
        })),

      toggleWidgetVisibility: (widgetId) =>
        set((state) => ({
          layout: {
            ...state.layout,
            widgets: state.layout.widgets.map((widget) =>
              widget.id === widgetId
                ? { ...widget, visible: !widget.visible }
                : widget
            ),
          },
        })),

      resetLayout: () => set({ layout: DEFAULT_LAYOUT }),
    }),
    {
      name: 'dashboard-layout',
    }
  )
);
