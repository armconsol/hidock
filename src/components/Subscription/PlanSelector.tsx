import { Card, Button, Badge, Space, Tag } from '@arco-design/web-react';
import { IconCheck } from '@arco-design/web-react/icon';
import { SubscriptionPlan } from '../../types/subscription';
import './PlanSelector.css';

interface PlanSelectorProps {
  plans: SubscriptionPlan[];
  currentPlanId?: string;
  onSelectPlan: (planId: string) => void;
}

export function PlanSelector({ plans, currentPlanId, onSelectPlan }: PlanSelectorProps) {
  const formatPrice = (price: number, currency: string) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currency,
    }).format(price);
  };

  const getPlanBadge = (plan: SubscriptionPlan) => {
    if (plan.type === 'trial') {
      return <Badge status="processing" text="Trial" />;
    }
    if (plan.type === 'yearly') {
      return <Badge status="success" text="Best Value" />;
    }
    return null;
  };

  return (
    <div className="plan-selector">
      <h2>Choose Your Plan</h2>
      <div className="plans-grid">
        {plans.map((plan) => {
          const isCurrentPlan = plan.id === currentPlanId;

          return (
            <Card
              key={plan.id}
              className={`plan-card ${isCurrentPlan ? 'current-plan' : ''}`}
              bordered
              hoverable={!isCurrentPlan}
            >
              <div className="plan-header">
                <Space direction="vertical" size={8}>
                  <div className="plan-name-row">
                    <h3>{plan.name}</h3>
                    {getPlanBadge(plan)}
                  </div>
                  {plan.type !== 'trial' && (
                    <div className="plan-price">
                      <span className="price-amount">{formatPrice(plan.price, plan.currency)}</span>
                      <span className="price-period">
                        {plan.type === 'monthly' ? '/month' : plan.type === 'yearly' ? '/year' : ''}
                      </span>
                    </div>
                  )}
                  {plan.transcriptionMinutes && (
                    <Tag color="arcoblue">
                      {plan.transcriptionMinutes} minutes
                    </Tag>
                  )}
                </Space>
              </div>

              <div className="plan-features">
                <ul>
                  {plan.features.map((feature, index) => (
                    <li key={index}>
                      <IconCheck style={{ color: 'var(--accent-color)' }} />
                      <span>{feature}</span>
                    </li>
                  ))}
                </ul>
              </div>

              <div className="plan-action">
                {isCurrentPlan ? (
                  <Button type="outline" disabled style={{ width: '100%' }}>
                    Current Plan
                  </Button>
                ) : (
                  <Button
                    type="primary"
                    onClick={() => onSelectPlan(plan.id)}
                    style={{ width: '100%' }}
                  >
                    {plan.type === 'trial' ? 'Start Trial' : 'Upgrade'}
                  </Button>
                )}
              </div>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
