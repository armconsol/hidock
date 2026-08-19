import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TranslationSettings } from './TranslationSettings';

describe('TranslationSettings Component', () => {
  const mockOnSave = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings form', () => {
    render(<TranslationSettings />);

    expect(screen.getByText(/default source language/i)).toBeInTheDocument();
    expect(screen.getByText(/default target language/i)).toBeInTheDocument();
    expect(screen.getByText(/auto-translate notes/i)).toBeInTheDocument();
  });

  it('displays current settings', () => {
    const settings = {
      defaultSourceLang: 'en',
      defaultTargetLang: 'es',
      autoTranslate: true,
    };

    render(<TranslationSettings settings={settings} />);

    expect(screen.getByText('English')).toBeInTheDocument();
    expect(screen.getByText('Spanish')).toBeInTheDocument();

    const autoTranslateSwitch = screen.getByRole('switch');
    expect(autoTranslateSwitch).toBeChecked();
  });

  it('allows changing default source language', async () => {
    const user = userEvent.setup();
    render(<TranslationSettings onSave={mockOnSave} />);

    const sourceSelect = screen.getAllByRole('combobox')[0];
    await user.click(sourceSelect);

    await waitFor(async () => {
      const frenchOption = screen.getByText('French');
      await user.click(frenchOption);
    });

    const saveButton = screen.getByRole('button', { name: /save/i });
    await user.click(saveButton);

    expect(mockOnSave).toHaveBeenCalledWith(
      expect.objectContaining({ defaultSourceLang: 'fr' })
    );
  });

  it('allows changing default target language', async () => {
    const user = userEvent.setup();
    render(<TranslationSettings onSave={mockOnSave} />);

    const targetSelect = screen.getAllByRole('combobox')[1];
    await user.click(targetSelect);

    await waitFor(async () => {
      const germanOption = screen.getByText('German');
      await user.click(germanOption);
    });

    const saveButton = screen.getByRole('button', { name: /save/i });
    await user.click(saveButton);

    expect(mockOnSave).toHaveBeenCalledWith(
      expect.objectContaining({ defaultTargetLang: 'de' })
    );
  });

  it('toggles auto-translate setting', async () => {
    const user = userEvent.setup();
    render(<TranslationSettings onSave={mockOnSave} />);

    const autoTranslateSwitch = screen.getByRole('switch');
    await user.click(autoTranslateSwitch);

    const saveButton = screen.getByRole('button', { name: /save/i });
    await user.click(saveButton);

    expect(mockOnSave).toHaveBeenCalledWith(
      expect.objectContaining({ autoTranslate: true })
    );
  });

  it('resets settings to defaults', async () => {
    const user = userEvent.setup();
    const settings = {
      defaultSourceLang: 'fr',
      defaultTargetLang: 'de',
      autoTranslate: true,
    };

    render(<TranslationSettings settings={settings} onSave={mockOnSave} />);

    const resetButton = screen.getByRole('button', { name: /reset/i });
    await user.click(resetButton);

    const saveButton = screen.getByRole('button', { name: /save/i });
    await user.click(saveButton);

    expect(mockOnSave).toHaveBeenCalledWith({
      defaultSourceLang: 'en',
      defaultTargetLang: 'es',
      autoTranslate: false,
    });
  });

  it('shows save confirmation message', async () => {
    const user = userEvent.setup();
    render(<TranslationSettings onSave={mockOnSave} />);

    // Make a change to enable the save button
    const autoTranslateSwitch = screen.getByRole('switch');
    await user.click(autoTranslateSwitch);

    const saveButton = screen.getByRole('button', { name: /save settings/i });
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockOnSave).toHaveBeenCalled();
    });
  });

  it('disables save button when no changes', () => {
    const settings = {
      defaultSourceLang: 'en',
      defaultTargetLang: 'es',
      autoTranslate: false,
    };

    render(<TranslationSettings settings={settings} />);

    const saveButton = screen.getByRole('button', { name: /save/i });
    expect(saveButton).toBeDisabled();
  });
});
