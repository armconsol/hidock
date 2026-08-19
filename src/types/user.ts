/**
 * User Profile and Security Types
 * Based on HiNotes API endpoints: /v1/user/*
 */

export type Region = 'us' | 'eu' | 'asia' | 'other';

export interface UserProfile {
  id: string;
  email: string;
  name: string;
  avatar?: string;
  region: Region;
  emailVerified: boolean;
  createdAt: string; // ISO date string
  updatedAt: string; // ISO date string
}

export interface UpdateProfileRequest {
  name?: string;
  region?: Region;
}

export interface ChangePasswordRequest {
  currentPassword: string;
  newPassword: string;
  confirmPassword: string;
}

export interface EmailVerificationRequest {
  code: string;
}

export interface AvatarUploadResponse {
  avatarUrl: string;
}
