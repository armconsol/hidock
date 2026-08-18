import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Login } from './Login';
import { useAuthStore } from '../../store/authStore';

// Mock the auth store
vi.mock('../../store/authStore', () => ({
  useAuthStore: vi.fn(),
}));

describe('Login Component', () => {
  const mockLoginWithEmail = vi.fn();
  const mockLoginWithOAuth = vi.fn();
  const mockClearError = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      loginWithEmail: mockLoginWithEmail,
      loginWithOAuth: mockLoginWithOAuth,
      isLoading: false,
      error: null,
      clearError: mockClearError,
    });
  });

  it('renders login component with OAuth buttons', () => {
    render(<Login />);

    expect(screen.getByText('Welcome to HiNotes')).toBeInTheDocument();
    expect(screen.getByText('Sign in to continue')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /continue with google/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /continue with apple/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /continue with email/i })).toBeInTheDocument();
  });

  it('handles Google OAuth login', async () => {
    const user = userEvent.setup();
    render(<Login />);

    const googleButton = screen.getByRole('button', { name: /continue with google/i });
    await user.click(googleButton);

    expect(mockLoginWithOAuth).toHaveBeenCalledWith('google');
  });

  it('handles Apple OAuth login', async () => {
    const user = userEvent.setup();
    render(<Login />);

    const appleButton = screen.getByRole('button', { name: /continue with apple/i });
    await user.click(appleButton);

    expect(mockLoginWithOAuth).toHaveBeenCalledWith('apple');
  });

  it('switches to email login mode', async () => {
    const user = userEvent.setup();
    render(<Login />);

    const emailButton = screen.getByRole('button', { name: /continue with email/i });
    await user.click(emailButton);

    await waitFor(() => {
      expect(screen.getByPlaceholderText('Enter your email')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Enter your password')).toBeInTheDocument();
    });
  });

  it('validates email format in email login form', async () => {
    const user = userEvent.setup();
    render(<Login />);

    // Switch to email mode
    const emailButton = screen.getByRole('button', { name: /continue with email/i });
    await user.click(emailButton);

    // Enter invalid email
    const emailInput = screen.getByPlaceholderText('Enter your email');
    await user.type(emailInput, 'invalid-email');

    const passwordInput = screen.getByPlaceholderText('Enter your password');
    await user.type(passwordInput, 'password123');

    const submitButton = screen.getByRole('button', { name: /sign in/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(screen.getByText(/invalid email format/i)).toBeInTheDocument();
    });

    expect(mockLoginWithEmail).not.toHaveBeenCalled();
  });

  it('validates password length in email login form', async () => {
    const user = userEvent.setup();
    render(<Login />);

    // Switch to email mode
    const emailButton = screen.getByRole('button', { name: /continue with email/i });
    await user.click(emailButton);

    // Enter valid email but short password
    const emailInput = screen.getByPlaceholderText('Enter your email');
    await user.type(emailInput, 'test@example.com');

    const passwordInput = screen.getByPlaceholderText('Enter your password');
    await user.type(passwordInput, '12345');

    const submitButton = screen.getByRole('button', { name: /sign in/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(screen.getByText(/password must be at least 6 characters/i)).toBeInTheDocument();
    });

    expect(mockLoginWithEmail).not.toHaveBeenCalled();
  });

  it('submits email login form with valid data', async () => {
    const user = userEvent.setup();
    render(<Login />);

    // Switch to email mode
    const emailButton = screen.getByRole('button', { name: /continue with email/i });
    await user.click(emailButton);

    // Enter valid credentials
    const emailInput = screen.getByPlaceholderText('Enter your email');
    await user.type(emailInput, 'test@example.com');

    const passwordInput = screen.getByPlaceholderText('Enter your password');
    await user.type(passwordInput, 'password123');

    const submitButton = screen.getByRole('button', { name: /sign in/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(mockLoginWithEmail).toHaveBeenCalledWith('test@example.com', 'password123');
    });
  });

  it('switches back from email mode to OAuth options', async () => {
    const user = userEvent.setup();
    render(<Login />);

    // Switch to email mode
    const emailButton = screen.getByRole('button', { name: /continue with email/i });
    await user.click(emailButton);

    await waitFor(() => {
      expect(screen.getByPlaceholderText('Enter your email')).toBeInTheDocument();
    });

    // Switch back
    const backButton = screen.getByRole('button', { name: /back to other options/i });
    await user.click(backButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /continue with google/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /continue with apple/i })).toBeInTheDocument();
    });
  });

  it('displays error message when error exists', async () => {
    (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      loginWithEmail: mockLoginWithEmail,
      loginWithOAuth: mockLoginWithOAuth,
      isLoading: false,
      error: 'Authentication failed',
      clearError: mockClearError,
    });

    render(<Login />);

    // Arco Message component renders asynchronously
    await waitFor(() => {
      expect(screen.getByText('Authentication failed')).toBeInTheDocument();
    });
  });

  it('clears error when error message is closed', async () => {
    const user = userEvent.setup();
    (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      loginWithEmail: mockLoginWithEmail,
      loginWithOAuth: mockLoginWithOAuth,
      isLoading: false,
      error: 'Authentication failed',
      clearError: mockClearError,
    });

    render(<Login />);

    // Wait for message to appear
    await waitFor(() => {
      expect(screen.getByText('Authentication failed')).toBeInTheDocument();
    });

    // Find the close button by class
    const closeButton = document.querySelector('.arco-alert-close-btn') as HTMLElement;
    await user.click(closeButton);

    expect(mockClearError).toHaveBeenCalled();
  });

  it('disables buttons when loading', () => {
    (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      loginWithEmail: mockLoginWithEmail,
      loginWithOAuth: mockLoginWithOAuth,
      isLoading: true,
      error: null,
      clearError: mockClearError,
    });

    render(<Login />);

    const googleButton = screen.getByRole('button', { name: /continue with google/i });
    const appleButton = screen.getByRole('button', { name: /continue with apple/i });

    expect(googleButton).toHaveClass('arco-btn-loading');
    expect(appleButton).toHaveClass('arco-btn-loading');
  });

  it('displays terms and privacy policy text', () => {
    render(<Login />);

    expect(
      screen.getByText(/by continuing, you agree to our terms of service and privacy policy/i)
    ).toBeInTheDocument();
  });
});
