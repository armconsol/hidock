import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TranslationPanel } from './TranslationPanel';

describe('TranslationPanel Component', () => {
  const mockOnTranslate = vi.fn();
  const mockOnCopy = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    // Mock clipboard API
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
    });
  });

  it('renders with default props', () => {
    render(<TranslationPanel />);

    expect(screen.getByText(/source language/i)).toBeInTheDocument();
    expect(screen.getByText(/target language/i)).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/enter text to translate/i)).toBeInTheDocument();
  });

  it('renders with initial text', () => {
    render(<TranslationPanel initialText="Hello world" />);

    expect(screen.getByDisplayValue('Hello world')).toBeInTheDocument();
  });

  it('displays translated text when provided', () => {
    render(
      <TranslationPanel
        initialText="Hello"
        translatedText="Hola"
        sourceLang="en"
        targetLang="es"
      />
    );

    expect(screen.getByText('Hola')).toBeInTheDocument();
  });

  it('shows confidence indicator when confidence is provided', () => {
    render(
      <TranslationPanel
        translatedText="Hola"
        confidence={0.95}
      />
    );

    expect(screen.getByText(/95%/i)).toBeInTheDocument();
  });

  it('allows changing source language', async () => {
    const user = userEvent.setup();
    render(<TranslationPanel onTranslate={mockOnTranslate} />);

    const sourceSelect = screen.getAllByRole('combobox')[0];
    await user.click(sourceSelect);

    await waitFor(() => {
      const spanishOption = screen.getByText('Spanish');
      user.click(spanishOption);
    });

    await waitFor(() => {
      expect(mockOnTranslate).toHaveBeenCalled();
    });
  });

  it('allows changing target language', async () => {
    const user = userEvent.setup();
    render(<TranslationPanel onTranslate={mockOnTranslate} />);

    const targetSelect = screen.getAllByRole('combobox')[1];
    await user.click(targetSelect);

    await waitFor(() => {
      const frenchOption = screen.getByText('French');
      user.click(frenchOption);
    });

    await waitFor(() => {
      expect(mockOnTranslate).toHaveBeenCalled();
    });
  });

  it('allows entering text to translate', async () => {
    const user = userEvent.setup();
    render(<TranslationPanel />);

    const textArea = screen.getByPlaceholderText(/enter text to translate/i);
    await user.type(textArea, 'Hello world');

    expect(textArea).toHaveValue('Hello world');
  });

  it('copies translated text to clipboard', async () => {
    const user = userEvent.setup();
    render(<TranslationPanel translatedText="Hola mundo" />);

    const copyButton = screen.getByRole('button', { name: /copy/i });
    await user.click(copyButton);

    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith('Hola mundo');
    });
  });

  it('disables copy button when no translated text', () => {
    render(<TranslationPanel />);

    const copyButton = screen.getByRole('button', { name: /copy/i });
    expect(copyButton).toBeDisabled();
  });

  it('shows loading state during translation', () => {
    render(<TranslationPanel isLoading={true} />);

    expect(screen.getByText(/translating/i)).toBeInTheDocument();
  });

  it('displays error message when translation fails', () => {
    render(<TranslationPanel error="Translation failed" />);

    expect(screen.getByText('Translation failed')).toBeInTheDocument();
  });

  it('swaps source and target languages', async () => {
    const user = userEvent.setup();
    const onSwap = vi.fn();
    render(
      <TranslationPanel
        sourceLang="en"
        targetLang="es"
        onSwapLanguages={onSwap}
      />
    );

    const swapButton = screen.getByRole('button', { name: /swap/i });
    await user.click(swapButton);

    expect(onSwap).toHaveBeenCalled();
  });

  it('displays side-by-side layout on desktop', () => {
    render(
      <TranslationPanel
        initialText="Hello"
        translatedText="Hola"
        layout="side-by-side"
      />
    );

    const container = screen.getByTestId('translation-panel');
    expect(container).toHaveClass('side-by-side');
  });

  it('displays stacked layout on mobile', () => {
    render(
      <TranslationPanel
        initialText="Hello"
        translatedText="Hola"
        layout="stacked"
      />
    );

    const container = screen.getByTestId('translation-panel');
    expect(container).toHaveClass('stacked');
  });

  it('shows character count for input text', () => {
    render(<TranslationPanel initialText="Hello world" />);

    expect(screen.getByText(/11.*characters/i)).toBeInTheDocument();
  });

  it('limits input text length', async () => {
    const user = userEvent.setup();
    const longText = 'a'.repeat(5001);
    render(<TranslationPanel maxLength={5000} />);

    const textArea = screen.getByPlaceholderText(/enter text to translate/i);
    await user.type(textArea, longText);

    expect(textArea).toHaveValue('a'.repeat(5000));
  });
});
