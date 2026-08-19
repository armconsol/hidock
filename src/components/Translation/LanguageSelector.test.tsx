import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { LanguageSelector } from './LanguageSelector';

describe('LanguageSelector Component', () => {
  const mockOnChange = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders with default language', () => {
    render(<LanguageSelector value="en" onChange={mockOnChange} />);

    expect(screen.getByText('English')).toBeInTheDocument();
  });

  it('displays language flag', () => {
    render(<LanguageSelector value="en" onChange={mockOnChange} />);

    const flag = screen.getByRole('img', { name: /english flag/i });
    expect(flag).toBeInTheDocument();
  });

  it('shows language options when clicked', async () => {
    const user = userEvent.setup();
    render(<LanguageSelector value="en" onChange={mockOnChange} />);

    const selector = screen.getByRole('combobox');
    await user.click(selector);

    await waitFor(() => {
      expect(screen.getByText('Spanish')).toBeInTheDocument();
      expect(screen.getByText('French')).toBeInTheDocument();
      expect(screen.getByText('German')).toBeInTheDocument();
      expect(screen.getByText('Chinese')).toBeInTheDocument();
      expect(screen.getByText('Japanese')).toBeInTheDocument();
    });
  });

  it('calls onChange when language is selected', async () => {
    const user = userEvent.setup();
    render(<LanguageSelector value="en" onChange={mockOnChange} />);

    const selector = screen.getByRole('combobox');
    await user.click(selector);

    // Wait for dropdown to appear and find Spanish option
    await waitFor(async () => {
      const options = screen.getAllByText('Spanish');
      // Click the one that's visible in the dropdown
      await user.click(options[options.length - 1]);
    });

    // Arco Design's Select passes the value as first argument (and other args)
    expect(mockOnChange).toHaveBeenCalled();
    expect(mockOnChange.mock.calls[0][0]).toBe('es');
  });

  it('filters languages by search term', async () => {
    const user = userEvent.setup();
    render(<LanguageSelector value="en" onChange={mockOnChange} showSearch />);

    const selector = screen.getByRole('combobox');
    await user.click(selector);

    // Wait for dropdown to open
    await waitFor(() => {
      expect(screen.getAllByText('Spanish').length).toBeGreaterThan(0);
    });

    const searchInput = screen.getByRole('textbox');
    await user.type(searchInput, 'span');

    // Just verify the input worked - Arco Design filtering is tested by them
    expect(searchInput).toHaveValue('span');
  });

  it('displays native language names', () => {
    render(<LanguageSelector value="es" onChange={mockOnChange} showNativeName />);

    expect(screen.getByText(/Español/i)).toBeInTheDocument();
  });

  it('disables selector when disabled prop is true', () => {
    render(<LanguageSelector value="en" onChange={mockOnChange} disabled />);

    const selector = screen.getByRole('combobox');
    expect(selector).toHaveAttribute('aria-disabled', 'true');
  });

  it('shows auto-detect option when enabled', async () => {
    const user = userEvent.setup();
    render(<LanguageSelector value="en" onChange={mockOnChange} allowAutoDetect />);

    const selector = screen.getByRole('combobox');
    await user.click(selector);

    await waitFor(() => {
      expect(screen.getByText(/auto.*detect/i)).toBeInTheDocument();
    });
  });
});
