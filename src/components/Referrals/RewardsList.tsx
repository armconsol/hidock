import { useState } from 'react';
import { Card, Table, Button, Tag, Space, message, Modal } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { GiftOutlined, ClockCircleOutlined } from '@ant-design/icons';
import { RewardItem } from '../../types/referral';
import './RewardsList.css';

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
          message.success('Reward redeemed successfully!');
        } catch (error) {
          message.error('Failed to redeem reward');
          console.error('Error redeeming reward:', error);
        } finally {
          setRedeemingId(null);
        }
      },
    });
  };

  const getRewardIcon = (type: string) => {
    switch (type) {
      case 'cash':
        return <GiftOutlined style={{ color: 'var(--color-success-6)' }} />;
      case 'minutes':
        return <ClockCircleOutlined style={{ color: 'var(--color-primary-6)' }} />;
      default:
        return null;
    }
  };

  const getRewardTypeLabel = (type: string) => {
    switch (type) {
      case 'cash':
        return 'Cash Payout';
      case 'minutes':
        return 'Transcription Minutes';
      default:
        return type;
    }
  };

  const getStatusTag = (status: string) => {
    const statusConfig: Record<string, { color: string; text: string }> = {
      available: { color: 'green', text: 'Available' },
      redeemed: { color: 'blue', text: 'Redeemed' },
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

  const availableColumns: ColumnsType<RewardItem> = [
    {
      title: 'Type',
      dataIndex: 'reward_type',
      render: (_value, record) => (
        <Space>
          {getRewardIcon(record.reward_type)}
          <span>{getRewardTypeLabel(record.reward_type)}</span>
        </Space>
      ),
    },
    {
      title: 'Amount',
      dataIndex: 'amount',
      render: (amount, record) => (
        <strong>
          {record.reward_type === 'cash' && '$'}
          {amount}
          {record.reward_type === 'minutes' && ' min'}
        </strong>
      ),
    },
    {
      title: 'Description',
      dataIndex: 'description',
    },
    {
      title: 'Expires',
      dataIndex: 'expires_at',
      render: (expiresAt) => formatExpiryDate(expiresAt),
    },
    {
      title: 'Status',
      dataIndex: 'status',
      render: (status) => getStatusTag(status),
    },
    {
      title: 'Action',
      render: (_value, record) => (
        <Button
          type="primary"
          size="small"
          loading={redeemingId === record.id}
          onClick={() => handleRedeem(record)}
        >
          Redeem
        </Button>
      ),
    },
  ];

  const historyColumns: ColumnsType<RewardItem> = [
    {
      title: 'Type',
      dataIndex: 'reward_type',
      render: (_value, record) => (
        <Space>
          {getRewardIcon(record.reward_type)}
          <span>{getRewardTypeLabel(record.reward_type)}</span>
        </Space>
      ),
    },
    {
      title: 'Amount',
      dataIndex: 'amount',
      render: (amount, record) => (
        <strong>
          {record.reward_type === 'cash' && '$'}
          {amount}
          {record.reward_type === 'minutes' && ' min'}
        </strong>
      ),
    },
    {
      title: 'Description',
      dataIndex: 'description',
    },
    {
      title: 'Redeemed',
      dataIndex: 'redeemed_at',
      render: (date) => (date ? new Date(date).toLocaleDateString() : '-'),
    },
    {
      title: 'Status',
      dataIndex: 'status',
      render: (status) => getStatusTag(status),
    },
  ];

  const availableRewards = rewards.filter((r) => r.status === 'available');
  const otherRewards = rewards.filter((r) => r.status !== 'available');

  return (
    <Card className="rewards-list-card" title="Your Rewards">
      {availableRewards.length === 0 && otherRewards.length === 0 ? (
        <div className="empty-rewards">
          <GiftOutlined style={{ fontSize: 48, color: 'var(--color-text-4)' }} />
          <p>No rewards yet. Start referring friends to earn rewards!</p>
        </div>
      ) : (
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          {/* Available Rewards */}
          {availableRewards.length > 0 && (
            <div className="rewards-section">
              <h3 className="section-title">Available to Redeem</h3>
              <Table
                columns={availableColumns}
                dataSource={availableRewards}
                loading={loading}
                pagination={false}
                rowKey="id"
                className="rewards-table"
              />
            </div>
          )}

          {/* Redeemed/Expired Rewards */}
          {otherRewards.length > 0 && (
            <div className="rewards-section">
              <h3 className="section-title">Reward History</h3>
              <Table
                columns={historyColumns}
                dataSource={otherRewards}
                loading={loading}
                pagination={{ pageSize: 5 }}
                rowKey="id"
                className="rewards-table"
              />
            </div>
          )}
        </Space>
      )}
    </Card>
  );
}
