import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import { LoginForm } from './LoginForm';
import { useAuthStore } from '../../store/authStore';

// Mock the auth store
vi.mock('../../store/authStore', () => ({
  useAuthStore: vi.fn(),
}));

// Mock react-router-dom
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe('LoginForm', () => {
  const mockLoginWithEmail = vi.fn();
  const mockLoginWithOAuth = vi.fn();
  const mockClearError = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockNavigate.mockClear();

    // Setup default mock implementation
    (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      loginWithEmail: mockLoginWithEmail,
      loginWithOAuth: mockLoginWithOAuth,
      isLoading: false,
      error: null,
      clearError: mockClearError,
    });
  });

  it('renders login form by default', () => {
    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    expect(screen.getByText('Welcome Back')).toBeInTheDocument();
    expect(
      screen.getByText('Sign in to continue to HiNotes')
    ).toBeInTheDocument();
  });

  it('renders OAuth buttons', () => {
    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    expect(screen.getByText('Continue with Google')).toBeInTheDocument();
    expect(screen.getByText('Continue with Apple')).toBeInTheDocument();
  });

  it('renders email/password form fields', () => {
    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    expect(screen.getByPlaceholderText('Enter your email')).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText('Enter your password')
    ).toBeInTheDocument();
  });

  it('switches to signup mode when toggle is clicked', async () => {
    const user = userEvent.setup();

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const toggleButton = screen.getByRole('button', { name: /sign up/i });
    await user.click(toggleButton);

    await waitFor(() => {
      expect(screen.getByText('Create Account')).toBeInTheDocument();
      expect(
        screen.getByText('Sign up to get started with HiNotes')
      ).toBeInTheDocument();
    });
  });

  it('shows name field in signup mode', async () => {
    const user = userEvent.setup();

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const toggleButton = screen.getByRole('button', { name: /sign up/i });
    await user.click(toggleButton);

    await waitFor(() => {
      expect(screen.getByPlaceholderText('Enter your name')).toBeInTheDocument();
    });
  });

  it('calls loginWithOAuth when Google button is clicked', async () => {
    const user = userEvent.setup();

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const googleButton = screen.getByRole('button', {
      name: /continue with google/i,
    });
    await user.click(googleButton);

    expect(mockLoginWithOAuth).toHaveBeenCalledWith('google');
  });

  it('calls loginWithOAuth when Apple button is clicked', async () => {
    const user = userEvent.setup();

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const appleButton = screen.getByRole('button', {
      name: /continue with apple/i,
    });
    await user.click(appleButton);

    expect(mockLoginWithOAuth).toHaveBeenCalledWith('apple');
  });

  it('validates email format', async () => {
    const user = userEvent.setup();

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const emailInput = screen.getByPlaceholderText('Enter your email');
    const submitButton = screen.getByRole('button', { name: /sign in/i });

    await user.type(emailInput, 'invalid-email');
    await user.click(submitButton);

    await waitFor(() => {
      expect(
        screen.getByText('Please enter a valid email address')
      ).toBeInTheDocument();
    });
  });

  it('validates password length', async () => {
    const user = userEvent.setup();

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const emailInput = screen.getByPlaceholderText('Enter your email');
    const passwordInput = screen.getByPlaceholderText('Enter your password');
    const submitButton = screen.getByRole('button', { name: /sign in/i });

    await user.type(emailInput, 'test@example.com');
    await user.type(passwordInput, 'short');
    await user.click(submitButton);

    await waitFor(() => {
      expect(
        screen.getByText('Password must be at least 8 characters')
      ).toBeInTheDocument();
    });
  });

  it('submits form with valid credentials', async () => {
    const user = userEvent.setup();
    mockLoginWithEmail.mockResolvedValueOnce(undefined);

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const emailInput = screen.getByPlaceholderText('Enter your email');
    const passwordInput = screen.getByPlaceholderText('Enter your password');
    const submitButton = screen.getByRole('button', { name: /sign in/i });

    await user.type(emailInput, 'test@example.com');
    await user.type(passwordInput, 'password123');
    await user.click(submitButton);

    await waitFor(() => {
      expect(mockLoginWithEmail).toHaveBeenCalledWith(
        'test@example.com',
        'password123'
      );
    });
  });

  it('displays error message when provided', () => {
    (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      loginWithEmail: mockLoginWithEmail,
      loginWithOAuth: mockLoginWithOAuth,
      isLoading: false,
      error: 'Invalid credentials',
      clearError: mockClearError,
    });

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    expect(screen.getByText('Invalid credentials')).toBeInTheDocument();
  });

  it('disables form when loading', () => {
    (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      loginWithEmail: mockLoginWithEmail,
      loginWithOAuth: mockLoginWithOAuth,
      isLoading: true,
      error: null,
      clearError: mockClearError,
    });

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const emailInput = screen.getByPlaceholderText('Enter your email');
    const passwordInput = screen.getByPlaceholderText('Enter your password');

    expect(emailInput).toBeDisabled();
    expect(passwordInput).toBeDisabled();
  });

  it('navigates to home on successful login', async () => {
    const user = userEvent.setup();
    mockLoginWithEmail.mockResolvedValueOnce(undefined);

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const emailInput = screen.getByPlaceholderText('Enter your email');
    const passwordInput = screen.getByPlaceholderText('Enter your password');
    const submitButton = screen.getByRole('button', { name: /sign in/i });

    await user.type(emailInput, 'test@example.com');
    await user.type(passwordInput, 'password123');
    await user.click(submitButton);

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/home');
    });
  });

  it('clears error when user attempts new login', async () => {
    const user = userEvent.setup();

    render(
      <BrowserRouter>
        <LoginForm />
      </BrowserRouter>
    );

    const googleButton = screen.getByRole('button', {
      name: /continue with google/i,
    });
    await user.click(googleButton);

    expect(mockClearError).toHaveBeenCalled();
  });
});
