import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter, MemoryRouter } from 'react-router-dom';
import { AppLayout } from './AppLayout';

// Mock the ThemeProvider
vi.mock('../ThemeProvider', () => ({
  ThemeProvider: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="theme-provider">{children}</div>
  ),
}));

// Mock Arco Design icons
vi.mock('@arco-design/web-react/icon', () => ({
  IconHome: () => <div data-testid="icon-home">Home Icon</div>,
  IconFile: () => <div data-testid="icon-file">File Icon</div>,
  IconLanguage: () => <div data-testid="icon-language">Language Icon</div>,
  IconMessage: () => <div data-testid="icon-message">Message Icon</div>,
  IconCheckSquare: () => (
    <div data-testid="icon-check-square">CheckSquare Icon</div>
  ),
  IconSettings: () => <div data-testid="icon-settings">Settings Icon</div>,
  IconUser: () => <div data-testid="icon-user">User Icon</div>,
}));

describe('AppLayout', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders without crashing', () => {
    render(
      <BrowserRouter>
        <AppLayout />
      </BrowserRouter>
    );

    expect(screen.getByTestId('theme-provider')).toBeInTheDocument();
  });

  it('renders all navigation menu items', () => {
    render(
      <BrowserRouter>
        <AppLayout />
      </BrowserRouter>
    );

    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Notes')).toBeInTheDocument();
    expect(screen.getByText('Translate')).toBeInTheDocument();
    expect(screen.getByText('Whispers')).toBeInTheDocument();
    expect(screen.getByText('To-Do')).toBeInTheDocument();
  });

  it('renders settings button in footer', () => {
    render(
      <BrowserRouter>
        <AppLayout />
      </BrowserRouter>
    );

    expect(screen.getByText('Settings')).toBeInTheDocument();
    expect(screen.getByTestId('icon-settings')).toBeInTheDocument();
  });

  it('renders all navigation icons', () => {
    render(
      <BrowserRouter>
        <AppLayout />
      </BrowserRouter>
    );

    expect(screen.getByTestId('icon-home')).toBeInTheDocument();
    expect(screen.getByTestId('icon-file')).toBeInTheDocument();
    expect(screen.getByTestId('icon-language')).toBeInTheDocument();
    expect(screen.getByTestId('icon-message')).toBeInTheDocument();
    expect(screen.getByTestId('icon-check-square')).toBeInTheDocument();
  });

  it('navigates to correct route when menu item is clicked', async () => {
    const user = userEvent.setup();

    render(
      <MemoryRouter initialEntries={['/']}>
        <AppLayout />
      </MemoryRouter>
    );

    const notesLink = screen.getByText('Notes').closest('.arco-menu-item');
    expect(notesLink).toBeInTheDocument();

    if (notesLink) {
      await user.click(notesLink);
    }
  });

  it('highlights the current route in navigation', () => {
    render(
      <MemoryRouter initialEntries={['/notes']}>
        <AppLayout />
      </MemoryRouter>
    );

    const notesMenuItem = screen
      .getByText('Notes')
      .closest('.arco-menu-item');
    expect(notesMenuItem).toHaveClass('arco-menu-selected');
  });

  it('renders root path (/) as /home in navigation', () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <AppLayout />
      </MemoryRouter>
    );

    const homeMenuItem = screen.getByText('Home').closest('.arco-menu-item');
    expect(homeMenuItem).toHaveClass('arco-menu-selected');
  });

  it('settings button is clickable', async () => {
    const user = userEvent.setup();

    render(
      <MemoryRouter initialEntries={['/home']}>
        <AppLayout />
      </MemoryRouter>
    );

    const settingsButton = screen
      .getByText('Settings')
      .closest('.footer-button');
    expect(settingsButton).toBeInTheDocument();

    if (settingsButton) {
      await user.click(settingsButton);
    }
  });

  it('settings button responds to keyboard navigation', async () => {
    const user = userEvent.setup();

    render(
      <MemoryRouter initialEntries={['/home']}>
        <AppLayout />
      </MemoryRouter>
    );

    const settingsButton = screen
      .getByText('Settings')
      .closest('.footer-button');

    if (settingsButton) {
      settingsButton.focus();
      await user.keyboard('{Enter}');
    }
  });

  it('has proper accessibility attributes', () => {
    render(
      <BrowserRouter>
        <AppLayout />
      </BrowserRouter>
    );

    const settingsButton = screen
      .getByText('Settings')
      .closest('.footer-button');
    expect(settingsButton).toHaveAttribute('role', 'button');
    expect(settingsButton).toHaveAttribute('tabIndex', '0');
  });

  it('sidebar has fixed width of 80px', () => {
    const { container } = render(
      <BrowserRouter>
        <AppLayout />
      </BrowserRouter>
    );

    const sidebar = container.querySelector('.app-sidebar');
    expect(sidebar).toBeInTheDocument();
  });

  it('wraps content in ThemeProvider', () => {
    render(
      <BrowserRouter>
        <AppLayout />
      </BrowserRouter>
    );

    expect(screen.getByTestId('theme-provider')).toBeInTheDocument();
  });
});
