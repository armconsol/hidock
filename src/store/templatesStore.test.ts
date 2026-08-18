import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useTemplatesStore } from './templatesStore';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

describe('templatesStore', () => {
  beforeEach(() => {
    // Reset store state
    useTemplatesStore.setState({
      templates: [],
      selectedTemplateId: null,
      filter: {},
      sortBy: 'updatedAt',
      sortOrder: 'desc',
      isLoading: false,
      error: null,
    });
    vi.clearAllMocks();
  });

  describe('loadTemplates', () => {
    it('should load templates successfully', async () => {
      const mockTemplates = [
        {
          id: 'template-1',
          title: 'Meeting Notes',
          content: '## Meeting Notes',
          isFavorite: false,
          isDefault: false,
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-01T00:00:00Z',
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockTemplates);

      const store = useTemplatesStore.getState();
      await store.loadTemplates();

      expect(invoke).toHaveBeenCalledWith('list_templates', {
        favoriteOnly: false,
        limit: 100,
        offset: 0,
      });
      expect(store.templates).toHaveLength(1);
      expect(store.templates[0].title).toBe('Meeting Notes');
      expect(store.isLoading).toBe(false);
    });

    it('should handle load error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed to load'));

      const store = useTemplatesStore.getState();
      await expect(store.loadTemplates()).rejects.toThrow();
      expect(store.error).toBe('Failed to load');
    });
  });

  describe('createTemplate', () => {
    it('should create a new template', async () => {
      const mockTemplate = {
        id: 'new-template',
        title: 'New Template',
        content: 'Content',
        isFavorite: false,
        isDefault: false,
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      };

      vi.mocked(invoke).mockResolvedValue(mockTemplate);

      const store = useTemplatesStore.getState();
      const result = await store.createTemplate('New Template', 'Content');

      expect(result.title).toBe('New Template');
      expect(store.templates).toHaveLength(1);
    });
  });

  describe('updateTemplate', () => {
    it('should update an existing template', async () => {
      const initialTemplate = {
        id: 'template-1',
        title: 'Original Title',
        content: 'Content',
        isFavorite: false,
        isDefault: false,
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-01'),
      };

      useTemplatesStore.setState({ templates: [initialTemplate] });

      const updatedTemplate = {
        ...initialTemplate,
        title: 'Updated Title',
      };

      vi.mocked(invoke).mockResolvedValue(updatedTemplate);

      const store = useTemplatesStore.getState();
      const result = await store.updateTemplate('template-1', { title: 'Updated Title' });

      expect(result.title).toBe('Updated Title');
      expect(store.templates[0].title).toBe('Updated Title');
    });
  });

  describe('deleteTemplate', () => {
    it('should delete a template', async () => {
      const template = {
        id: 'template-1',
        title: 'Test Template',
        content: 'Content',
        isFavorite: false,
        isDefault: false,
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-01'),
      };

      useTemplatesStore.setState({ templates: [template] });

      vi.mocked(invoke).mockResolvedValue(undefined);

      const store = useTemplatesStore.getState();
      await store.deleteTemplate('template-1');

      expect(store.templates).toHaveLength(0);
    });
  });

  describe('toggleFavorite', () => {
    it('should toggle favorite status', async () => {
      const template = {
        id: 'template-1',
        title: 'Test Template',
        content: 'Content',
        isFavorite: false,
        isDefault: false,
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-01'),
      };

      useTemplatesStore.setState({ templates: [template] });

      const toggledTemplate = { ...template, isFavorite: true };
      vi.mocked(invoke).mockResolvedValue(toggledTemplate);

      const store = useTemplatesStore.getState();
      const result = await store.toggleFavorite('template-1');

      expect(result.isFavorite).toBe(true);
      expect(store.templates[0].isFavorite).toBe(true);
    });
  });

  describe('setAsDefault', () => {
    it('should set template as default and unset others', async () => {
      const templates = [
        {
          id: 'template-1',
          title: 'Template 1',
          content: 'Content 1',
          isFavorite: false,
          isDefault: true,
          createdAt: new Date('2024-01-01'),
          updatedAt: new Date('2024-01-01'),
        },
        {
          id: 'template-2',
          title: 'Template 2',
          content: 'Content 2',
          isFavorite: false,
          isDefault: false,
          createdAt: new Date('2024-01-01'),
          updatedAt: new Date('2024-01-01'),
        },
      ];

      useTemplatesStore.setState({ templates });

      const updatedTemplate = { ...templates[1], isDefault: true };
      vi.mocked(invoke).mockResolvedValue(updatedTemplate);

      const store = useTemplatesStore.getState();
      await store.setAsDefault('template-2');

      expect(store.templates[0].isDefault).toBe(false);
      expect(store.templates[1].isDefault).toBe(true);
    });
  });

  describe('getFilteredTemplates', () => {
    beforeEach(() => {
      const templates = [
        {
          id: 'template-1',
          title: 'Meeting Notes',
          content: 'Content for meetings',
          isFavorite: true,
          isDefault: false,
          createdAt: new Date('2024-01-01'),
          updatedAt: new Date('2024-01-03'),
        },
        {
          id: 'template-2',
          title: 'Daily Standup',
          content: 'Content for standups',
          isFavorite: false,
          isDefault: false,
          createdAt: new Date('2024-01-02'),
          updatedAt: new Date('2024-01-02'),
        },
      ];

      useTemplatesStore.setState({ templates });
    });

    it('should filter by search query', () => {
      const store = useTemplatesStore.getState();
      store.setFilter({ searchQuery: 'meeting' });

      const filtered = store.getFilteredTemplates();
      expect(filtered).toHaveLength(1);
      expect(filtered[0].title).toBe('Meeting Notes');
    });

    it('should filter by favorite only', () => {
      const store = useTemplatesStore.getState();
      store.setFilter({ favoriteOnly: true });

      const filtered = store.getFilteredTemplates();
      expect(filtered).toHaveLength(1);
      expect(filtered[0].isFavorite).toBe(true);
    });

    it('should sort by title ascending', () => {
      const store = useTemplatesStore.getState();
      store.setSorting('title', 'asc');

      const filtered = store.getFilteredTemplates();
      expect(filtered[0].title).toBe('Daily Standup');
      expect(filtered[1].title).toBe('Meeting Notes');
    });

    it('should sort by updatedAt descending', () => {
      const store = useTemplatesStore.getState();
      store.setSorting('updatedAt', 'desc');

      const filtered = store.getFilteredTemplates();
      expect(filtered[0].title).toBe('Meeting Notes'); // More recent
      expect(filtered[1].title).toBe('Daily Standup');
    });
  });
});
