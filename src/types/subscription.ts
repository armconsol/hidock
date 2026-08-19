/**
 * Subscription Types
 * Based on HiNotes API endpoints: /v1/subscribers, /v1/receipts, /v1/payment/rc/portal
 */

export interface SubscriptionPlan {
  id: string;
  name: string;
  type: 'monthly' | 'yearly' | 'quota' | 'trial';
  price: number;
  currency: string;
  features: string[];
  transcriptionMinutes?: number; // Minutes included or remaining
  isActive: boolean;
}

export interface SubscriberInfo {
  userId: string;
  plan: SubscriptionPlan;
  expiryDate: string; // ISO date string
  autoRenew: boolean;
  trialStatus: 'eligible' | 'active' | 'expired' | 'claimed';
  trialExpiryDate?: string; // ISO date string
  usageStats: {
    minutesUsed: number;
    minutesRemaining: number;
    totalMinutesAllotted: number;
    resetDate?: string; // ISO date string for monthly plans
  };
}

export interface Receipt {
  id: string;
  date: string; // ISO date string
  amount: number;
  currency: string;
  planName: string;
  status: 'completed' | 'pending' | 'failed' | 'refunded';
  receiptUrl?: string;
  transactionId: string;
}

export interface TrialStatus {
  eligible: boolean;
  active: boolean;
  daysRemaining?: number;
  expiryDate?: string; // ISO date string
}

export interface BillingPortalUrl {
  url: string;
  expiresAt: string; // ISO date string
}
