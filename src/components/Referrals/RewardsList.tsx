import { useState } from 'react';
import { Card, Table, Button, Tag, Space, Message, Modal } from '@arco-design/web-react';
import { IconGift, IconClockCircle, IconCurrency } from '@arco-design/web-react/icon';
import { RewardItem } from '../../types/referral';
import './RewardsList.css';

const { Column } = Table;

interface RewardsListProps {
  rewards: RewardItem[];
  onRedeemReward: (rewardId: string, amount: number) => Promise<void>;
  loading?: boolean;
}

export function RewardsList({ rewards, onRedeemReward, loading = false }: RewardsListProps) {
  const [redeemingId, setRedeemingId] = useState<string | null>(null);

  const handleRedeem = async (reward: RewardItem) => {
    Modal.confirm({
      title: 'Redeem Reward',
      content: `Are you sure you want to redeem ${reward.amount} ${reward.reward_type}?`,
      onOk: async () => {
        setRedeemingId(reward.id);
        try {
          await onRedeemReward(reward.id, reward.amount);
          Message.success('Reward redeemed successfully!');
        } catch (error) {
          Message.error('Failed to redeem reward');
          console.error('Error redeeming reward:', error);
        } finally {
          setRedeemingId(null);
        }
      },
    });
  };

  const getRewardIcon = (type: RewardItem['reward_type']) => {
    switch (type) {
      case 'minutes':
        return <IconClockCircle />;
      case 'cash':
        return <IconCurrency />;
      case 'credit':
        return <IconGift />;
    }
  };

  const getRewardTypeLabel = (type: RewardItem['reward_type']) => {
    switch (type) {
      case 'minutes':
        return 'Transcription Minutes';
      case 'cash':
        return 'Cash Reward';
      case 'credit':
        return 'Account Credit';
    }
  };

  const getStatusTag = (status: RewardItem['status']) => {
    const statusConfig = {
      available: { color: 'green', text: 'Available' },
      redeemed: { color: 'arcoblue', text: 'Redeemed' },
      expired: { color: 'red', text: 'Expired' },
      pending: { color: 'orange', text: 'Pending' },
    };

    const config = statusConfig[status];
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  const formatExpiryDate = (dateString?: string) => {
    if (!dateString) return 'Never';
    const date = new Date(dateString);
    const now = new Date();
    const daysUntilExpiry = Math.ceil((date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));

    if (daysUntilExpiry < 0) return 'Expired';
    if (daysUntilExpiry === 0) return 'Today';
    if (daysUntilExpiry === 1) return 'Tomorrow';
    if (daysUntilExpiry <= 7) return `${daysUntilExpiry} days`;

    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const availableRewards = rewards.filter((r) => r.status === 'available');
  const otherRewards = rewards.filter((r) => r.status !== 'available');

  return (
    <Card className="rewards-list-card" title="Your Rewards">
      {availableRewards.length === 0 && otherRewards.length === 0 ? (
        <div className="empty-rewards">
          <IconGift style={{ fontSize: 48, color: 'var(--color-text-4)' }} />
          <p>No rewards yet. Start referring friends to earn rewards!</p>
        </div>
      ) : (
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          {/* Available Rewards */}
          {availableRewards.length > 0 && (
            <div className="rewards-section">
              <h3 className="section-title">Available to Redeem</h3>
              <Table
                data={availableRewards}
                loading={loading}
                pagination={false}
                rowKey="id"
                className="rewards-table"
              >
                <Column
                  title="Type"
                  dataIndex="reward_type"
                  render={(_, record: RewardItem) => (
                    <Space>
                      {getRewardIcon(record.reward_type)}
                      <span>{getRewardTypeLabel(record.reward_type)}</span>
                    </Space>
                  )}
                />
                <Column
                  title="Amount"
                  dataIndex="amount"
                  render={(amount, record: RewardItem) => (
                    <strong>
                      {record.reward_type === 'cash' && '$'}
                      {amount}
                      {record.reward_type === 'minutes' && ' min'}
                    </strong>
                  )}
                />
                <Column
                  title="Description"
                  dataIndex="description"
                />
                <Column
                  title="Expires"
                  dataIndex="expires_at"
                  render={(expiresAt) => formatExpiryDate(expiresAt)}
                />
                <Column
                  title="Status"
                  dataIndex="status"
                  render={(status) => getStatusTag(status)}
                />
                <Column
                  title="Action"
                  render={(_, record: RewardItem) => (
                    <Button
                      type="primary"
                      size="small"
                      loading={redeemingId === record.id}
                      onClick={() => handleRedeem(record)}
                    >
                      Redeem
                    </Button>
                  )}
                />
              </Table>
            </div>
          )}

          {/* Redeemed/Expired Rewards */}
          {otherRewards.length > 0 && (
            <div className="rewards-section">
              <h3 className="section-title">Reward History</h3>
              <Table
                data={otherRewards}
                loading={loading}
                pagination={{ pageSize: 5 }}
                rowKey="id"
                className="rewards-table history"
              >
                <Column
                  title="Type"
                  dataIndex="reward_type"
                  render={(_, record: RewardItem) => (
                    <Space>
                      {getRewardIcon(record.reward_type)}
                      <span>{getRewardTypeLabel(record.reward_type)}</span>
                    </Space>
                  )}
                />
                <Column
                  title="Amount"
                  dataIndex="amount"
                  render={(amount, record: RewardItem) => (
                    <span>
                      {record.reward_type === 'cash' && '$'}
                      {amount}
                      {record.reward_type === 'minutes' && ' min'}
                    </span>
                  )}
                />
                <Column
                  title="Description"
                  dataIndex="description"
                />
                <Column
                  title="Date"
                  dataIndex="updated_at"
                  render={(date) =>
                    new Date(date).toLocaleDateString('en-US', {
                      year: 'numeric',
                      month: 'short',
                      day: 'numeric',
                    })
                  }
                />
                <Column
                  title="Status"
                  dataIndex="status"
                  render={(status) => getStatusTag(status)}
                />
              </Table>
            </div>
          )}
        </Space>
      )}
    </Card>
  );
}
