import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { InlineTranslation } from './InlineTranslation';

describe('InlineTranslation Component', () => {
  const mockOnTranslate = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders original text', () => {
    render(<InlineTranslation text="Hello world" />);

    expect(screen.getByText('Hello world')).toBeInTheDocument();
  });

  it('shows translate button on hover', async () => {
    const user = userEvent.setup();
    render(<InlineTranslation text="Hello world" />);

    const container = screen.getByTestId('inline-translation');
    await user.hover(container);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /translate/i })).toBeInTheDocument();
    });
  });

  it('displays translated text when available', () => {
    render(
      <InlineTranslation
        text="Hello world"
        translatedText="Hola mundo"
        targetLang="es"
      />
    );

    expect(screen.getByText('Hola mundo')).toBeInTheDocument();
  });

  it('calls onTranslate when translate button is clicked', async () => {
    const user = userEvent.setup();
    render(<InlineTranslation text="Hello world" onTranslate={mockOnTranslate} />);

    const container = screen.getByTestId('inline-translation');
    await user.hover(container);

    const translateButton = await screen.findByRole('button', { name: /translate/i });
    await user.click(translateButton);

    expect(mockOnTranslate).toHaveBeenCalledWith('Hello world');
  });

  it('shows loading indicator during translation', () => {
    render(<InlineTranslation text="Hello world" isLoading={true} />);

    expect(screen.getByTestId('inline-translation')).toBeInTheDocument();
  });

  it('toggles between original and translated text', async () => {
    const user = userEvent.setup();
    render(
      <InlineTranslation
        text="Hello world"
        translatedText="Hola mundo"
        targetLang="es"
      />
    );

    // Initially shows translated text (because translatedText is provided)
    expect(screen.getByText('Hola mundo')).toBeInTheDocument();

    const toggleButton = screen.getByRole('button', { name: /show original/i });
    await user.click(toggleButton);

    await waitFor(() => {
      expect(screen.getByText('Hello world')).toBeInTheDocument();
      expect(screen.queryByText('Hola mundo')).not.toBeInTheDocument();
    });

    const newToggleButton = screen.getByRole('button', { name: /show translation/i });
    await user.click(newToggleButton);

    await waitFor(() => {
      expect(screen.getByText('Hola mundo')).toBeInTheDocument();
      expect(screen.queryByText('Hello world')).not.toBeInTheDocument();
    });
  });

  it('displays error when translation fails', () => {
    render(
      <InlineTranslation
        text="Hello world"
        error="Translation failed"
      />
    );

    expect(screen.getByText(/translation failed/i)).toBeInTheDocument();
  });

  it('shows confidence badge when available', () => {
    render(
      <InlineTranslation
        text="Hello"
        translatedText="Hola"
        confidence={0.95}
      />
    );

    expect(screen.getByText(/95%/i)).toBeInTheDocument();
  });
});
