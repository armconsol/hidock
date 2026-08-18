import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import type { Template, TemplateFilter, TemplateSortBy, SortOrder } from '../types/templates';

interface TemplatesState {
  templates: Template[];
  selectedTemplateId: string | null;
  filter: TemplateFilter;
  sortBy: TemplateSortBy;
  sortOrder: SortOrder;
  isLoading: boolean;
  error: string | null;

  // Actions
  loadTemplates: (favoriteOnly?: boolean) => Promise<void>;
  getTemplate: (id: string) => Promise<Template | null>;
  getDefaultTemplate: () => Promise<Template | null>;
  createTemplate: (
    title: string,
    content: string,
    isFavorite?: boolean,
    isDefault?: boolean
  ) => Promise<Template>;
  updateTemplate: (
    id: string,
    updates: {
      title?: string;
      content?: string;
      isFavorite?: boolean;
      isDefault?: boolean;
    }
  ) => Promise<Template>;
  deleteTemplate: (id: string) => Promise<void>;
  toggleFavorite: (id: string) => Promise<Template>;
  setAsDefault: (id: string) => Promise<Template>;
  selectTemplate: (id: string | null) => void;
  setFilter: (filter: TemplateFilter) => void;
  setSorting: (sortBy: TemplateSortBy, sortOrder: SortOrder) => void;
  getFilteredTemplates: () => Template[];
}

export const useTemplatesStore = create<TemplatesState>()(
  persist(
    (set, get) => ({
      templates: [],
      selectedTemplateId: null,
      filter: {},
      sortBy: 'updatedAt',
      sortOrder: 'desc',
      isLoading: false,
      error: null,

      loadTemplates: async (favoriteOnly = false) => {
        set({ isLoading: true, error: null });
        try {
          const templates = await invoke<Template[]>('list_templates', {
            favoriteOnly,
            limit: 100,
            offset: 0,
          });

          // Convert date strings to Date objects
          const parsedTemplates = templates.map((t) => ({
            ...t,
            createdAt: new Date(t.createdAt),
            updatedAt: new Date(t.updatedAt),
            syncedAt: t.syncedAt ? new Date(t.syncedAt) : undefined,
          }));

          set({ templates: parsedTemplates, isLoading: false });
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to load templates',
            isLoading: false,
          });
          throw error;
        }
      },

      getTemplate: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
          const template = await invoke<Template | null>('get_template', {
            templateId: id,
          });

          set({ isLoading: false });

          if (!template) return null;

          return {
            ...template,
            createdAt: new Date(template.createdAt),
            updatedAt: new Date(template.updatedAt),
            syncedAt: template.syncedAt ? new Date(template.syncedAt) : undefined,
          };
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to get template',
            isLoading: false,
          });
          throw error;
        }
      },

      getDefaultTemplate: async () => {
        set({ isLoading: true, error: null });
        try {
          const template = await invoke<Template | null>('get_default_template');

          set({ isLoading: false });

          if (!template) return null;

          return {
            ...template,
            createdAt: new Date(template.createdAt),
            updatedAt: new Date(template.updatedAt),
            syncedAt: template.syncedAt ? new Date(template.syncedAt) : undefined,
          };
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to get default template',
            isLoading: false,
          });
          throw error;
        }
      },

      createTemplate: async (
        title: string,
        content: string,
        isFavorite = false,
        isDefault = false
      ) => {
        set({ isLoading: true, error: null });
        try {
          const template = await invoke<Template>('create_template', {
            id: crypto.randomUUID(),
            title,
            content,
            isFavorite,
            isDefault,
          });

          const parsedTemplate = {
            ...template,
            createdAt: new Date(template.createdAt),
            updatedAt: new Date(template.updatedAt),
            syncedAt: template.syncedAt ? new Date(template.syncedAt) : undefined,
          };

          set((state) => ({
            templates: [parsedTemplate, ...state.templates],
            isLoading: false,
          }));

          return parsedTemplate;
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to create template',
            isLoading: false,
          });
          throw error;
        }
      },

      updateTemplate: async (id: string, updates) => {
        set({ isLoading: true, error: null });
        try {
          const template = await invoke<Template>('update_template', {
            id,
            ...updates,
          });

          const parsedTemplate = {
            ...template,
            createdAt: new Date(template.createdAt),
            updatedAt: new Date(template.updatedAt),
            syncedAt: template.syncedAt ? new Date(template.syncedAt) : undefined,
          };

          set((state) => ({
            templates: state.templates.map((t) => (t.id === id ? parsedTemplate : t)),
            isLoading: false,
          }));

          return parsedTemplate;
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to update template',
            isLoading: false,
          });
          throw error;
        }
      },

      deleteTemplate: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
          await invoke('delete_template', { templateId: id });

          set((state) => ({
            templates: state.templates.filter((t) => t.id !== id),
            selectedTemplateId: state.selectedTemplateId === id ? null : state.selectedTemplateId,
            isLoading: false,
          }));
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to delete template',
            isLoading: false,
          });
          throw error;
        }
      },

      toggleFavorite: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
          const template = await invoke<Template>('toggle_template_favorite', { id });

          const parsedTemplate = {
            ...template,
            createdAt: new Date(template.createdAt),
            updatedAt: new Date(template.updatedAt),
            syncedAt: template.syncedAt ? new Date(template.syncedAt) : undefined,
          };

          set((state) => ({
            templates: state.templates.map((t) => (t.id === id ? parsedTemplate : t)),
            isLoading: false,
          }));

          return parsedTemplate;
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to toggle favorite',
            isLoading: false,
          });
          throw error;
        }
      },

      setAsDefault: async (id: string) => {
        set({ isLoading: true, error: null });
        try {
          const template = await invoke<Template>('set_template_default', { id });

          const parsedTemplate = {
            ...template,
            createdAt: new Date(template.createdAt),
            updatedAt: new Date(template.updatedAt),
            syncedAt: template.syncedAt ? new Date(template.syncedAt) : undefined,
          };

          // Update all templates: unset others, set this one
          set((state) => ({
            templates: state.templates.map((t) =>
              t.id === id ? parsedTemplate : { ...t, isDefault: false }
            ),
            isLoading: false,
          }));

          return parsedTemplate;
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to set default template',
            isLoading: false,
          });
          throw error;
        }
      },

      selectTemplate: (id) => {
        set({ selectedTemplateId: id });
      },

      setFilter: (filter) => {
        set({ filter });
      },

      setSorting: (sortBy, sortOrder) => {
        set({ sortBy, sortOrder });
      },

      getFilteredTemplates: () => {
        const { templates, filter, sortBy, sortOrder } = get();

        let filtered = templates.filter((template) => {
          // Filter by search query
          if (filter.searchQuery) {
            const query = filter.searchQuery.toLowerCase();
            if (
              !template.title.toLowerCase().includes(query) &&
              !template.content.toLowerCase().includes(query)
            ) {
              return false;
            }
          }

          // Filter by favorite only
          if (filter.favoriteOnly && !template.isFavorite) {
            return false;
          }

          return true;
        });

        // Sort
        filtered.sort((a, b) => {
          let aValue: string | number | Date;
          let bValue: string | number | Date;

          if (sortBy === 'title') {
            aValue = a.title.toLowerCase();
            bValue = b.title.toLowerCase();
          } else {
            aValue = new Date(a[sortBy]).getTime();
            bValue = new Date(b[sortBy]).getTime();
          }

          if (sortOrder === 'asc') {
            return aValue > bValue ? 1 : -1;
          } else {
            return aValue < bValue ? 1 : -1;
          }
        });

        return filtered;
      },
    }),
    {
      name: 'templates-storage',
      partialize: (state) => ({
        filter: state.filter,
        sortBy: state.sortBy,
        sortOrder: state.sortOrder,
      }),
    }
  )
);
