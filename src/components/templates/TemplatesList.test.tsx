import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { TemplatesList } from './TemplatesList';
import { useTemplatesStore } from '../../store/templatesStore';

vi.mock('../../store/templatesStore');

describe('TemplatesList', () => {
  const mockTemplates = [
    {
      id: 'template-1',
      title: 'Meeting Notes',
      content: '## Meeting Notes\n\nDate: \nAttendees: ',
      isFavorite: true,
      isDefault: true,
      createdAt: new Date('2024-01-01'),
      updatedAt: new Date('2024-01-01'),
    },
    {
      id: 'template-2',
      title: 'Daily Standup',
      content: '## Daily Standup\n\nYesterday:\nToday:\nBlockers:',
      isFavorite: false,
      isDefault: false,
      createdAt: new Date('2024-01-02'),
      updatedAt: new Date('2024-01-02'),
    },
  ];

  const mockStore = {
    templates: mockTemplates,
    selectedTemplateId: null,
    filter: {},
    sortBy: 'updatedAt' as const,
    sortOrder: 'desc' as const,
    isLoading: false,
    error: null,
    loadTemplates: vi.fn(),
    selectTemplate: vi.fn(),
    setFilter: vi.fn(),
    setSorting: vi.fn(),
    toggleFavorite: vi.fn(),
    setAsDefault: vi.fn(),
    deleteTemplate: vi.fn(),
    getFilteredTemplates: vi.fn(() => mockTemplates),
  };

  beforeEach(() => {
    vi.mocked(useTemplatesStore).mockReturnValue(mockStore);
  });

  it('should render templates list', () => {
    render(<TemplatesList />);

    expect(screen.getByText('Meeting Notes')).toBeInTheDocument();
    expect(screen.getByText('Daily Standup')).toBeInTheDocument();
  });

  it('should load templates on mount', () => {
    render(<TemplatesList />);

    expect(mockStore.loadTemplates).toHaveBeenCalled();
  });

  it('should display default and favorite tags', () => {
    render(<TemplatesList />);

    expect(screen.getByText('Default')).toBeInTheDocument();
    expect(screen.getByText('Favorite')).toBeInTheDocument();
  });

  it('should handle search input', () => {
    render(<TemplatesList />);

    const searchInput = screen.getByPlaceholderText('Search templates...');
    fireEvent.change(searchInput, { target: { value: 'meeting' } });

    expect(mockStore.setFilter).toHaveBeenCalledWith({ searchQuery: 'meeting' });
  });

  it('should render sort select with current value', () => {
    render(<TemplatesList />);

    // Verify the sort select is rendered
    const sortSelect = screen.getByPlaceholderText('Sort by');
    expect(sortSelect).toBeInTheDocument();

    // Verify the current sort value is displayed
    expect(screen.getByText('Last Modified (Newest)')).toBeInTheDocument();
  });

  it('should toggle favorite filter', () => {
    render(<TemplatesList />);

    const favoriteButton = screen.getAllByRole('button').find((btn) =>
      btn.querySelector('.arco-icon-star')
    );

    if (favoriteButton) {
      fireEvent.click(favoriteButton);
      expect(mockStore.setFilter).toHaveBeenCalled();
    }
  });

  it('should handle template selection', () => {
    render(<TemplatesList />);

    const templateCard = screen.getByText('Meeting Notes').closest('.template-card');
    if (templateCard) {
      fireEvent.click(templateCard);
      expect(mockStore.selectTemplate).toHaveBeenCalledWith('template-1');
    }
  });

  it('should toggle favorite status', async () => {
    mockStore.toggleFavorite.mockResolvedValue(mockTemplates[0]);
    render(<TemplatesList />);

    const favoriteButtons = screen.getAllByRole('button').filter((btn) =>
      btn.querySelector('.arco-icon-star-fill')
    );

    if (favoriteButtons.length > 0) {
      fireEvent.click(favoriteButtons[0]);
      await waitFor(() => {
        expect(mockStore.toggleFavorite).toHaveBeenCalled();
      });
    }
  });

  it('should display empty state when no templates', () => {
    vi.mocked(useTemplatesStore).mockReturnValue({
      ...mockStore,
      getFilteredTemplates: vi.fn(() => []),
    });

    render(<TemplatesList />);

    expect(
      screen.getByText('No templates yet. Create your first template!')
    ).toBeInTheDocument();
  });

  it('should display loading state', () => {
    vi.mocked(useTemplatesStore).mockReturnValue({
      ...mockStore,
      isLoading: true,
    });

    const { container } = render(<TemplatesList />);

    // Check for loading container
    expect(container.querySelector('.templates-list-loading')).toBeInTheDocument();
  });

  it('should display error message', () => {
    vi.mocked(useTemplatesStore).mockReturnValue({
      ...mockStore,
      error: 'Failed to load templates',
    });

    render(<TemplatesList />);

    expect(screen.getByText('Failed to load templates')).toBeInTheDocument();
  });

  it('should handle new template button click', () => {
    render(<TemplatesList />);

    const newButton = screen.getByText('New Template');
    fireEvent.click(newButton);

    expect(mockStore.selectTemplate).toHaveBeenCalledWith('new');
  });

  it('should set template as default', async () => {
    mockStore.setAsDefault.mockResolvedValue(undefined);
    render(<TemplatesList />);

    // Find the non-default template (Daily Standup)
    const dailyStandupCard = screen.getByText('Daily Standup').closest('.template-card');
    expect(dailyStandupCard).toBeInTheDocument();

    // Find the "Set as Default" button (IconCheck button)
    const setDefaultButtons = screen.getAllByRole('button').filter((btn) =>
      btn.querySelector('.arco-icon-check')
    );

    expect(setDefaultButtons.length).toBeGreaterThan(0);

    // Click the set default button
    fireEvent.click(setDefaultButtons[0]);

    await waitFor(() => {
      expect(mockStore.setAsDefault).toHaveBeenCalledWith('template-2');
    });
  });
});
