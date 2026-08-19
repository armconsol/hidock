/**
 * Referral Types
 * Based on HiNotes API endpoints: /v1/referral/*
 */

export interface ReferralCode {
  id: string;
  user_id: string;
  code: string;
  created_at: string; // ISO date string
  expires_at?: string; // ISO date string
  is_active: boolean;
}

export interface ReferralUsage {
  id: number;
  code_id: string;
  referred_user_id: string;
  referrer_user_id: string;
  applied_at: string; // ISO date string
  reward_points: number;
  reward_credits?: number;
  reward_subscription_days?: number;
}

export interface ReferralStats {
  total_referrals: number;
  successful_conversions: number;
  total_points_earned: number;
  total_credits_earned: number;
  total_subscription_days_earned: number;
  active_codes_count: number;
}

export interface ReferralOverview {
  referral_code: string;
  referral_link: string;
  total_referrals: number;
  successful_signups: number;
  pending_rewards: number;
  total_earnings: number;
  currency: string;
}

export interface RewardItem {
  id: string;
  reward_type: 'minutes' | 'cash' | 'credit';
  amount: number;
  description: string;
  expires_at?: string; // ISO date string
  status: 'available' | 'redeemed' | 'expired' | 'pending';
  created_at: string; // ISO date string
  updated_at: string; // ISO date string
}

export interface RewardHistory {
  id: string;
  reward_id: string;
  action: 'redeemed' | 'expired' | 'payout_requested' | 'payout_completed' | 'payout_failed';
  points_used: number;
  occurred_at: string; // ISO date string
  details?: string;
}

export interface RewardsOverview {
  available_rewards: number;
  total_minutes_earned: number;
  total_cash_earned: number;
  pending_payouts: number;
  paypal_connected: boolean;
  paypal_email?: string;
}

export interface ReferralMessageTemplate {
  subject: string;
  body: string;
  short_message: string;
  share_url: string;
}

export interface PayPalConnection {
  email: string;
  connected_at: string; // ISO date string
  status: 'active' | 'disconnected';
}
