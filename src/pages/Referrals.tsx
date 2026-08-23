import { useState, useEffect } from 'react';
import { Space, message, Spin, Alert } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { ReferralLink } from '../components/Referrals/ReferralLink';
import { ReferralStats } from '../components/Referrals/ReferralStats';
import { RewardsList } from '../components/Referrals/RewardsList';
import { PayoutSettings } from '../components/Referrals/PayoutSettings';
import {
  ReferralStats as ReferralStatsType,
  ReferralCode,
  RewardItem,
  PayPalConnection,
} from '../types/referral';
import './Referrals.css';

export function ReferralsPage() {
  const [loading, setLoading] = useState(true);
  const [referralCode, setReferralCode] = useState<ReferralCode | null>(null);
  const [stats, setStats] = useState<ReferralStatsType | null>(null);
  const [rewards, setRewards] = useState<RewardItem[]>([]);
  const [paypalConnection, setPaypalConnection] = useState<PayPalConnection | undefined>();
  const [availableCash, setAvailableCash] = useState(0);

  // Mock user ID - Replace with actual auth context
  const userId = 'user-123';
  const minimumPayout = 10.0;

  useEffect(() => {
    loadReferralData();
  }, []);

  const loadReferralData = async () => {
    setLoading(true);
    try {
      // Load referral code
      await loadReferralCode();

      // Load stats
      await loadStats();

      // Load rewards
      await loadRewards();

      // Mock PayPal connection - Replace with actual API call
      // API endpoint: GET /v1/referral/rewards-overview
      setPaypalConnection(undefined); // Not connected by default
      setAvailableCash(0);
    } catch (error) {
      message.error('Failed to load referral data');
      console.error('Error loading referral data:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadReferralCode = async () => {
    try {
      // Check if user already has a referral code
      const codes = await invoke<ReferralCode[]>('get_user_referral_codes', {
        userId,
      });

      if (codes.length > 0) {
        setReferralCode(codes[0]);
      } else {
        // Create new referral code
        const newCode = await invoke<ReferralCode>('create_referral_code', {
          userId,
          expiresAt: null, // No expiry
        });
        setReferralCode(newCode);
      }
    } catch (error) {
      console.error('Error loading referral code:', error);
      throw error;
    }
  };

  const loadStats = async () => {
    try {
      const referralStats = await invoke<ReferralStatsType>('get_referral_stats', {
        userId,
      });
      setStats(referralStats);
    } catch (error) {
      console.error('Error loading stats:', error);
      throw error;
    }
  };

  const loadRewards = async () => {
    try {
      // Load all rewards (available and history)
      const allRewards = await invoke<RewardItem[]>('list_rewards', {
        statusFilter: null, // Get all rewards
      });
      setRewards(allRewards);

      // Calculate available cash
      const cashRewards = allRewards.filter(
        (r) => r.reward_type === 'cash' && r.status === 'available'
      );
      const totalCash = cashRewards.reduce((sum, r) => sum + r.amount, 0);
      setAvailableCash(totalCash);
    } catch (error) {
      console.error('Error loading rewards:', error);
      throw error;
    }
  };

  const handleRedeemReward = async (rewardId: string, amount: number) => {
    try {
      await invoke('redeem_reward', {
        rewardId,
        pointsToUse: amount,
      });
      await loadRewards(); // Refresh rewards list
      await loadStats(); // Refresh stats
    } catch (error) {
      console.error('Error redeeming reward:', error);
      throw error;
    }
  };

  const handleConnectPayPal = async (email: string, _authCode: string) => {
    try {
      // Mock API call - Replace with actual implementation
      // API endpoint: POST /v1/referral/paypal/connect
      // const response = await fetch('https://hinotes.hidock.com/v1/referral/paypal/connect', {
      //   method: 'POST',
      //   headers: {
      //     'Content-Type': 'application/json',
      //     'Authorization': `Bearer ${token}`,
      //   },
      //   body: JSON.stringify({ email, authCode }),
      // });

      // Mock successful connection
      setPaypalConnection({
        email,
        connected_at: new Date().toISOString(),
        status: 'active',
      });
    } catch (error) {
      console.error('Error connecting PayPal:', error);
      throw error;
    }
  };

  const handleDisconnectPayPal = async () => {
    try {
      // Mock API call - Replace with actual implementation
      // API endpoint: POST /v1/referral/paypal/disconnect
      setPaypalConnection(undefined);
    } catch (error) {
      console.error('Error disconnecting PayPal:', error);
      throw error;
    }
  };

  const handleRequestPayout = async (amount: number) => {
    try {
      if (!paypalConnection) {
        throw new Error('PayPal not connected');
      }

      await invoke('request_payout', {
        amount,
        paypalEmail: paypalConnection.email,
        threshold: minimumPayout,
      });

      await loadRewards(); // Refresh rewards
    } catch (error) {
      console.error('Error requesting payout:', error);
      throw error;
    }
  };

  const handleShareReferral = (platform: string) => {
    console.log(`Shared referral link via ${platform}`);
    // Optional: Track sharing analytics
  };

  if (loading) {
    return (
      <div className="referrals-page loading">
        <Spin size="large" />
      </div>
    );
  }

  if (!referralCode || !stats) {
    return (
      <div className="referrals-page">
        <h1>Referral Program</h1>
        <Alert
          type="error"
          message="Failed to load referral information. Please try again later."
        />
      </div>
    );
  }

  const referralLink = `https://hinotes.hidock.com/signup?ref=${referralCode.code}`;

  return (
    <div className="referrals-page">
      <div className="referrals-header">
        <h1>Referral Program</h1>
        <p className="header-subtitle">
          Share HiNotes with friends and earn rewards for every successful referral!
        </p>
      </div>

      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        {/* Referral Link & Sharing */}
        <ReferralLink
          referralCode={referralCode.code}
          referralLink={referralLink}
          onShare={handleShareReferral}
        />

        {/* Statistics */}
        <ReferralStats stats={stats} />

        {/* Payout Settings */}
        <PayoutSettings
          paypalConnection={paypalConnection}
          availableCash={availableCash}
          minimumPayout={minimumPayout}
          onConnectPayPal={handleConnectPayPal}
          onDisconnectPayPal={handleDisconnectPayPal}
          onRequestPayout={handleRequestPayout}
        />

        {/* Rewards List */}
        <RewardsList
          rewards={rewards}
          onRedeemReward={handleRedeemReward}
          loading={false}
        />
      </Space>
    </div>
  );
}
