import { useState, useEffect } from 'react';
import { Card, Button, Space, Statistic, Progress, Divider, Message, Spin } from '@arco-design/web-react';
import { IconClockCircle, IconCalendar, IconLink } from '@arco-design/web-react/icon';
import { PlanSelector } from '../components/Subscription/PlanSelector';
import { BillingHistory } from '../components/Subscription/BillingHistory';
import { TrialBanner } from '../components/Subscription/TrialBanner';
import { SubscriberInfo, Receipt, SubscriptionPlan } from '../types/subscription';
import './Subscription.css';

export function SubscriptionPage() {
  const [loading, setLoading] = useState(true);
  const [subscriberInfo, setSubscriberInfo] = useState<SubscriberInfo | null>(null);
  const [receipts, setReceipts] = useState<Receipt[]>([]);
  const [availablePlans, setAvailablePlans] = useState<SubscriptionPlan[]>([]);

  useEffect(() => {
    loadSubscriptionData();
  }, []);

  const loadSubscriptionData = async () => {
    setLoading(true);
    try {
      // Mock data - Replace with actual API calls
      // API endpoints: GET /v1/subscribers, GET /v1/receipts

      const mockSubscriberInfo: SubscriberInfo = {
        userId: 'user123',
        plan: {
          id: 'monthly-standard',
          name: 'Monthly Standard',
          type: 'monthly',
          price: 9.99,
          currency: 'USD',
          features: [
            '120 transcription minutes/month',
            'Real-time translation',
            'Cloud sync across devices',
            'Export to multiple formats',
          ],
          transcriptionMinutes: 120,
          isActive: true,
        },
        expiryDate: new Date(Date.now() + 15 * 24 * 60 * 60 * 1000).toISOString(), // 15 days from now
        autoRenew: true,
        trialStatus: 'expired',
        usageStats: {
          minutesUsed: 45,
          minutesRemaining: 75,
          totalMinutesAllotted: 120,
          resetDate: new Date(Date.now() + 15 * 24 * 60 * 60 * 1000).toISOString(),
        },
      };

      const mockReceipts: Receipt[] = [
        {
          id: 'receipt1',
          date: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
          amount: 9.99,
          currency: 'USD',
          planName: 'Monthly Standard',
          status: 'completed',
          transactionId: 'TXN123456789',
        },
        {
          id: 'receipt2',
          date: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString(),
          amount: 9.99,
          currency: 'USD',
          planName: 'Monthly Standard',
          status: 'completed',
          transactionId: 'TXN987654321',
        },
      ];

      const mockPlans: SubscriptionPlan[] = [
        {
          id: 'monthly-basic',
          name: 'Basic',
          type: 'monthly',
          price: 4.99,
          currency: 'USD',
          features: [
            '60 transcription minutes/month',
            'Basic transcription',
            'Cloud sync',
          ],
          transcriptionMinutes: 60,
          isActive: false,
        },
        {
          id: 'monthly-standard',
          name: 'Standard',
          type: 'monthly',
          price: 9.99,
          currency: 'USD',
          features: [
            '120 transcription minutes/month',
            'Real-time translation',
            'Cloud sync across devices',
            'Export to multiple formats',
          ],
          transcriptionMinutes: 120,
          isActive: true,
        },
        {
          id: 'yearly-premium',
          name: 'Premium',
          type: 'yearly',
          price: 99.99,
          currency: 'USD',
          features: [
            '1800 transcription minutes/year',
            'Priority processing',
            'Advanced AI features',
            'Real-time translation',
            'Unlimited exports',
            'Priority support',
          ],
          transcriptionMinutes: 1800,
          isActive: false,
        },
      ];

      setSubscriberInfo(mockSubscriberInfo);
      setReceipts(mockReceipts);
      setAvailablePlans(mockPlans);
    } catch (error) {
      Message.error('Failed to load subscription data');
      console.error('Error loading subscription:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleClaimTrial = async () => {
    try {
      // API call: POST /v1/user/trial/claim
      Message.success('Trial claimed successfully!');
      await loadSubscriptionData();
    } catch (error) {
      Message.error('Failed to claim trial');
      console.error('Error claiming trial:', error);
    }
  };

  const handleSelectPlan = async (planId: string) => {
    try {
      // This would typically open RevenueCat billing portal
      // API call: GET /v1/payment/rc/portal
      Message.info('Redirecting to billing portal...');
      console.log('Selected plan:', planId);
    } catch (error) {
      Message.error('Failed to open billing portal');
      console.error('Error selecting plan:', error);
    }
  };

  const handleDownloadReceipt = async (receiptId: string) => {
    try {
      // Download receipt PDF/file
      Message.success('Downloading receipt...');
      console.log('Download receipt:', receiptId);
    } catch (error) {
      Message.error('Failed to download receipt');
      console.error('Error downloading receipt:', error);
    }
  };

  const handleOpenBillingPortal = async () => {
    try {
      // API call: GET /v1/payment/rc/portal
      Message.info('Opening RevenueCat billing portal...');
      // window.open(portalUrl, '_blank');
    } catch (error) {
      Message.error('Failed to open billing portal');
      console.error('Error opening billing portal:', error);
    }
  };

  if (loading) {
    return (
      <div className="subscription-page loading">
        <Spin size={40} />
      </div>
    );
  }

  if (!subscriberInfo) {
    return (
      <div className="subscription-page">
        <h1>Subscription</h1>
        <Card>
          <p>Failed to load subscription information. Please try again later.</p>
        </Card>
      </div>
    );
  }

  const usagePercent = (subscriberInfo.usageStats.minutesUsed / subscriberInfo.usageStats.totalMinutesAllotted) * 100;
  const daysUntilRenewal = Math.ceil((new Date(subscriberInfo.expiryDate).getTime() - Date.now()) / (1000 * 60 * 60 * 24));

  return (
    <div className="subscription-page">
      <div className="subscription-header">
        <h1>Subscription</h1>
        <Button
          type="outline"
          icon={<IconLink />}
          onClick={handleOpenBillingPortal}
        >
          Manage Billing
        </Button>
      </div>

      <TrialBanner
        trialStatus={{
          eligible: subscriberInfo.trialStatus === 'eligible',
          active: subscriberInfo.trialStatus === 'active',
          daysRemaining: subscriberInfo.trialStatus === 'active' && subscriberInfo.trialExpiryDate
            ? Math.ceil((new Date(subscriberInfo.trialExpiryDate).getTime() - Date.now()) / (1000 * 60 * 60 * 24))
            : undefined,
          expiryDate: subscriberInfo.trialExpiryDate,
        }}
        onClaimTrial={handleClaimTrial}
      />

      <Card className="current-subscription-card">
        <h2>Current Subscription</h2>
        <div className="subscription-info">
          <Space size="large" wrap>
            <div className="info-item">
              <label>Plan</label>
              <div className="plan-name">{subscriberInfo.plan.name}</div>
              <div className="plan-type">
                {subscriberInfo.plan.type === 'monthly' ? 'Monthly' : subscriberInfo.plan.type === 'yearly' ? 'Yearly' : 'One-time'}
              </div>
            </div>
            <Divider type="vertical" style={{ height: '60px' }} />
            <div className="info-item">
              <label>
                <IconCalendar style={{ marginRight: 4 }} />
                Renewal Date
              </label>
              <div className="renewal-date">
                {new Date(subscriberInfo.expiryDate).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric',
                })}
              </div>
              <div className="days-remaining">
                {daysUntilRenewal} days remaining
              </div>
            </div>
            <Divider type="vertical" style={{ height: '60px' }} />
            <div className="info-item">
              <label>Auto-Renewal</label>
              <div className={`auto-renew-status ${subscriberInfo.autoRenew ? 'active' : 'inactive'}`}>
                {subscriberInfo.autoRenew ? 'Enabled' : 'Disabled'}
              </div>
            </div>
          </Space>
        </div>
      </Card>

      <Card className="usage-stats-card">
        <h2>
          <IconClockCircle style={{ marginRight: 8 }} />
          Transcription Usage
        </h2>
        <div className="usage-content">
          <Space size="large" wrap>
            <Statistic
              title="Minutes Used"
              value={subscriberInfo.usageStats.minutesUsed}
              suffix={`/ ${subscriberInfo.usageStats.totalMinutesAllotted}`}
              precision={0}
            />
            <Statistic
              title="Minutes Remaining"
              value={subscriberInfo.usageStats.minutesRemaining}
              precision={0}
            />
          </Space>
          <div className="usage-progress">
            <Progress
              percent={usagePercent}
              status={usagePercent > 90 ? 'warning' : 'normal'}
              formatText={(percent) => `${percent?.toFixed(0)}% used`}
            />
          </div>
          {subscriberInfo.usageStats.resetDate && (
            <p className="reset-info">
              Usage resets on {new Date(subscriberInfo.usageStats.resetDate).toLocaleDateString('en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
              })}
            </p>
          )}
        </div>
      </Card>

      <PlanSelector
        plans={availablePlans}
        currentPlanId={subscriberInfo.plan.id}
        onSelectPlan={handleSelectPlan}
      />

      <BillingHistory
        receipts={receipts}
        onDownloadReceipt={handleDownloadReceipt}
      />
    </div>
  );
}
