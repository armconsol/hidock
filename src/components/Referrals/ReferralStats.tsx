import { Card, Statistic, Space, Row, Col, Divider } from 'antd';
import {
  UserOutlined,
  UserAddOutlined,
  GiftOutlined,
  ClockCircleOutlined,
  CalendarOutlined,
} from '@ant-design/icons';
import { ReferralStats as ReferralStatsType } from '../../types/referral';
import './ReferralStats.css';

interface ReferralStatsProps {
  stats: ReferralStatsType;
}

export function ReferralStats({ stats }: ReferralStatsProps) {
  const conversionRate =
    stats.total_referrals > 0
      ? ((stats.successful_conversions / stats.total_referrals) * 100).toFixed(1)
      : '0';

  return (
    <Card className="referral-stats-card" title="Referral Statistics">
      <Row gutter={[16, 16]}>
        {/* Total Referrals */}
        <Col xs={24} sm={12} md={8} lg={6}>
          <div className="stat-item">
            <Statistic
              title="Total Referrals"
              value={stats.total_referrals}
              prefix={<UserOutlined style={{ color: 'var(--color-primary-6)' }} />}
            />
          </div>
        </Col>

        {/* Successful Conversions */}
        <Col xs={24} sm={12} md={8} lg={6}>
          <div className="stat-item">
            <Statistic
              title="Successful Signups"
              value={stats.successful_conversions}
              prefix={<UserAddOutlined style={{ color: 'var(--color-success-6)' }} />}
            />
            <div className="stat-subtitle">
              {conversionRate}% conversion rate
            </div>
          </div>
        </Col>

        {/* Total Points */}
        <Col xs={24} sm={12} md={8} lg={6}>
          <div className="stat-item">
            <Statistic
              title="Total Points Earned"
              value={stats.total_points_earned}
              prefix={<GiftOutlined style={{ color: 'var(--color-warning-6)' }} />}
            />
          </div>
        </Col>

        {/* Active Codes */}
        <Col xs={24} sm={12} md={8} lg={6}>
          <div className="stat-item">
            <Statistic
              title="Active Codes"
              value={stats.active_codes_count}
              prefix={<UserAddOutlined style={{ color: 'var(--color-primary-6)' }} />}
            />
          </div>
        </Col>
      </Row>

      <Divider />

      {/* Rewards Breakdown */}
      <div className="rewards-breakdown">
        <h3 className="section-title">Rewards Earned</h3>
        <Space size="large" wrap>
          <div className="reward-stat">
            <ClockCircleOutlined style={{ fontSize: 24, color: 'var(--color-primary-6)' }} />
            <div className="reward-stat-content">
              <div className="reward-stat-label">Transcription Minutes</div>
              <div className="reward-stat-value">
                {stats.total_subscription_days_earned} minutes
              </div>
            </div>
          </div>

          <div className="reward-stat">
            <GiftOutlined style={{ fontSize: 24, color: 'var(--color-success-6)' }} />
            <div className="reward-stat-content">
              <div className="reward-stat-label">Credits</div>
              <div className="reward-stat-value">
                ${stats.total_credits_earned.toFixed(2)}
              </div>
            </div>
          </div>

          <div className="reward-stat">
            <CalendarOutlined style={{ fontSize: 24, color: 'var(--color-warning-6)' }} />
            <div className="reward-stat-content">
              <div className="reward-stat-label">Subscription Days</div>
              <div className="reward-stat-value">
                {stats.total_subscription_days_earned} days
              </div>
            </div>
          </div>
        </Space>
      </div>
    </Card>
  );
}
