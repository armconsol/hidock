import { Table, Button, Badge, Empty } from '@arco-design/web-react';
import { IconDownload } from '@arco-design/web-react/icon';
import { Receipt } from '../../types/subscription';
import './BillingHistory.css';

interface BillingHistoryProps {
  receipts: Receipt[];
  onDownloadReceipt: (receiptId: string) => void;
}

export function BillingHistory({ receipts, onDownloadReceipt }: BillingHistoryProps) {
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const formatAmount = (amount: number, currency: string) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currency,
    }).format(amount);
  };

  const getStatusBadge = (status: Receipt['status']) => {
    const statusConfig = {
      completed: { status: 'success' as const, text: 'Completed' },
      pending: { status: 'processing' as const, text: 'Pending' },
      failed: { status: 'error' as const, text: 'Failed' },
      refunded: { status: 'warning' as const, text: 'Refunded' },
    };

    const config = statusConfig[status];
    return <Badge status={config.status} text={config.text} />;
  };

  const columns = [
    {
      title: 'Date',
      dataIndex: 'date',
      key: 'date',
      render: (date: string) => formatDate(date),
      width: 120,
    },
    {
      title: 'Plan',
      dataIndex: 'planName',
      key: 'planName',
    },
    {
      title: 'Amount',
      dataIndex: 'amount',
      key: 'amount',
      render: (amount: number, record: Receipt) => formatAmount(amount, record.currency),
      width: 120,
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status: Receipt['status']) => getStatusBadge(status),
      width: 120,
    },
    {
      title: 'Transaction ID',
      dataIndex: 'transactionId',
      key: 'transactionId',
      ellipsis: true,
    },
    {
      title: 'Action',
      key: 'action',
      width: 120,
      render: (record: Receipt) => (
        <Button
          type="text"
          icon={<IconDownload />}
          onClick={() => onDownloadReceipt(record.id)}
          disabled={!record.receiptUrl}
        >
          Download
        </Button>
      ),
    },
  ];

  if (receipts.length === 0) {
    return (
      <div className="billing-history">
        <h2>Billing History</h2>
        <Empty description="No billing history available" />
      </div>
    );
  }

  return (
    <div className="billing-history">
      <div className="billing-history-header">
        <h2>Billing History</h2>
        <p className="billing-history-subtitle">
          View and download your past transactions
        </p>
      </div>
      <Table
        columns={columns}
        data={receipts}
        rowKey="id"
        pagination={{
          pageSize: 10,
          showTotal: (total) => `Total ${total} receipts`,
        }}
        stripe
      />
    </div>
  );
}
