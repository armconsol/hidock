import { Alert, Button, Space, Progress } from 'antd';
import { ClockCircleOutlined, CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';
import { TrialStatus } from '../../types/subscription';
import './TrialBanner.css';

interface TrialBannerProps {
  trialStatus: TrialStatus;
  onClaimTrial: () => void;
}

export function TrialBanner({ trialStatus, onClaimTrial }: TrialBannerProps) {
  // Don't show banner if not eligible and no active trial
  if (!trialStatus.eligible && !trialStatus.active) {
    return null;
  }

  // Trial is eligible but not claimed
  if (trialStatus.eligible && !trialStatus.active) {
    return (
      <Alert
        type="info"
        icon={<CheckCircleOutlined />}
        message="Free Trial Available"
        description={
          <div className="trial-banner-content">
            <p>Start your free trial to unlock premium features and transcription minutes.</p>
            <Button type="primary" onClick={onClaimTrial}>
              Claim Free Trial
            </Button>
          </div>
        }
        className="trial-banner"
        closable={false}
      />
    );
  }

  // Trial is active
  if (trialStatus.active && trialStatus.daysRemaining !== undefined) {
    const totalTrialDays = 30; // Assuming 30-day trial
    const progressPercent = ((totalTrialDays - trialStatus.daysRemaining) / totalTrialDays) * 100;
    const isExpiringSoon = trialStatus.daysRemaining <= 3;

    return (
      <Alert
        type={isExpiringSoon ? 'warning' : 'success'}
        icon={isExpiringSoon ? <CloseCircleOutlined /> : <ClockCircleOutlined />}
        message={isExpiringSoon ? 'Trial Ending Soon' : 'Trial Active'}
        description={
          <div className="trial-banner-content">
            <Space direction="vertical" size={12} style={{ width: '100%' }}>
              <p>
                {isExpiringSoon
                  ? `Your trial expires in ${trialStatus.daysRemaining} day${trialStatus.daysRemaining === 1 ? '' : 's'}. Upgrade now to continue using premium features.`
                  : `You have ${trialStatus.daysRemaining} day${trialStatus.daysRemaining === 1 ? '' : 's'} remaining in your trial period.`}
              </p>
              <Progress
                percent={progressPercent}
                status={isExpiringSoon ? 'exception' : 'normal'}
                showInfo={false}
              />
              {trialStatus.expiryDate && (
                <p className="trial-expiry-date">
                  Expires on: {new Date(trialStatus.expiryDate).toLocaleDateString('en-US', {
                    year: 'numeric',
                    month: 'long',
                    day: 'numeric',
                  })}
                </p>
              )}
            </Space>
          </div>
        }
        className="trial-banner"
        closable={false}
      />
    );
  }

  return null;
}
