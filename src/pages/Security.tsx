import { useState, useEffect } from 'react';
import {
  Card,
  Form,
  Input,
  Button,
  message,
  Divider,
  Space,
  Modal,
  Alert,
} from 'antd';
import {
  LockOutlined,
  MailOutlined,
  DeleteOutlined,
  CheckCircleFilled,
  CloseCircleFilled,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import { ChangePasswordRequest } from '../types/user';
import './Security.css';

const FormItem = Form.Item;

export function SecurityPage() {
  const [passwordForm] = Form.useForm();
  const navigate = useNavigate();
  const { user, logout } = useAuthStore();
  const [changingPassword, setChangingPassword] = useState(false);
  const [emailVerified, setEmailVerified] = useState(false);
  const [sendingCode, setSendingCode] = useState(false);
  const [verifyingCode, setVerifyingCode] = useState(false);
  const [verificationCode, setVerificationCode] = useState('');
  const [codeSent, setCodeSent] = useState(false);
  const [deleteModalVisible, setDeleteModalVisible] = useState(false);
  const [deleteConfirmText, setDeleteConfirmText] = useState('');
  const [deleting, setDeleting] = useState(false);

  useEffect(() => {
    loadSecurityInfo();
  }, []);

  const loadSecurityInfo = async () => {
    try {
      // TODO: Replace with actual API call
      // API endpoint: POST /v1/user/info

      // Mock data
      setEmailVerified(true);
    } catch (error) {
      console.error('Error loading security info:', error);
    }
  };

  const handleChangePassword = async () => {
    try {
      await passwordForm.validateFields();
      const values = passwordForm.getFieldsValue() as ChangePasswordRequest;

      if (values.newPassword !== values.confirmPassword) {
        message.error('New passwords do not match');
        return;
      }

      if (values.currentPassword === values.newPassword) {
        message.error('New password must be different from current password');
        return;
      }

      setChangingPassword(true);
      try {
        // TODO: Replace with actual API call
        // API endpoint: POST /v1/user/password/update

        await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call

        message.success('Password changed successfully');
        passwordForm.resetFields();
      } catch (error) {
        message.error('Failed to change password');
        console.error('Error changing password:', error);
      } finally {
        setChangingPassword(false);
      }
    } catch (error) {
      // Form validation error
      console.error('Form validation error:', error);
    }
  };

  const handleSendVerificationCode = async () => {
    setSendingCode(true);
    try {
      // TODO: Replace with actual API call
      // API endpoint: POST /v1/user/email/verification/send

      await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call

      setCodeSent(true);
      message.success('Verification code sent to your email');
    } catch (error) {
      message.error('Failed to send verification code');
      console.error('Error sending verification code:', error);
    } finally {
      setSendingCode(false);
    }
  };

  const handleVerifyEmail = async () => {
    if (!verificationCode || verificationCode.length !== 6) {
      message.error('Please enter a valid 6-digit code');
      return;
    }

    setVerifyingCode(true);
    try {
      // TODO: Replace with actual API call
      // API endpoint: POST /v1/user/email/verification/verify

      await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call

      setEmailVerified(true);
      setCodeSent(false);
      setVerificationCode('');
      message.success('Email verified successfully');
    } catch (error) {
      message.error('Invalid verification code');
      console.error('Error verifying email:', error);
    } finally {
      setVerifyingCode(false);
    }
  };

  const handleDeleteAccount = async () => {
    if (deleteConfirmText !== 'DELETE') {
      message.error('Please type DELETE to confirm');
      return;
    }

    setDeleting(true);
    try {
      // TODO: Replace with actual API call
      // API endpoint: POST /v1/user/delete

      await new Promise(resolve => setTimeout(resolve, 1500)); // Simulate API call

      message.success('Account deleted successfully');
      setDeleteModalVisible(false);

      // Logout and redirect to login
      await logout();
      navigate('/login');
    } catch (error) {
      message.error('Failed to delete account');
      console.error('Error deleting account:', error);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div className="security-page">
      <h1>Security Settings</h1>

      {/* Change Password */}
      <Card className="security-card" title="Change Password">
        <Form
          form={passwordForm}
          layout="vertical"
          className="password-form"
        >
          <FormItem
            label="Current Password"
            name="currentPassword"
            rules={[{ required: true, message: 'Current password is required' }]}
          >
            <Input.Password placeholder="Enter current password" />
          </FormItem>

          <FormItem
            label="New Password"
            name="newPassword"
            rules={[
              { required: true, message: 'New password is required' },
              { min: 8, message: 'Password must be at least 8 characters' },
              {
                pattern: /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/,
                message: 'Password must contain uppercase, lowercase, and number',
              },
            ]}
          >
            <Input.Password placeholder="Enter new password" />
          </FormItem>

          <FormItem
            label="Confirm New Password"
            name="confirmPassword"
            rules={[{ required: true, message: 'Please confirm new password' }]}
          >
            <Input.Password placeholder="Confirm new password" />
          </FormItem>

          <Button
            type="primary"
            icon={<LockOutlined />}
            onClick={handleChangePassword}
            loading={changingPassword}
            style={{ marginTop: 8 }}
          >
            Change Password
          </Button>
        </Form>
      </Card>

      {/* Email Verification */}
      <Card className="security-card" title="Email Verification">
        <div className="email-verification-section">
          <div className="verification-status">
            <Space size="small">
              <MailOutlined style={{ fontSize: 18 }} />
              <span className="email">{user?.email}</span>
              {emailVerified ? (
                <span className="verified-badge">
                  <CheckCircleFilled style={{ marginRight: 4 }} />
                  Verified
                </span>
              ) : (
                <span className="unverified-badge">
                  <CloseCircleFilled style={{ marginRight: 4 }} />
                  Not Verified
                </span>
              )}
            </Space>
          </div>

          {!emailVerified && (
            <>
              <Divider />
              {!codeSent ? (
                <div className="verification-actions">
                  <p>Verify your email address to secure your account and enable all features.</p>
                  <Button
                    type="primary"
                    onClick={handleSendVerificationCode}
                    loading={sendingCode}
                  >
                    Send Verification Code
                  </Button>
                </div>
              ) : (
                <div className="verification-code-input">
                  <p>Enter the 6-digit code sent to your email:</p>
                  <Space direction="vertical" style={{ width: '100%' }}>
                    <Input
                      placeholder="Enter 6-digit code"
                      value={verificationCode}
                      onChange={(e) => setVerificationCode(e.target.value)}
                      maxLength={6}
                      style={{ width: 200 }}
                    />
                    <Space>
                      <Button
                        type="primary"
                        onClick={handleVerifyEmail}
                        loading={verifyingCode}
                        disabled={verificationCode.length !== 6}
                      >
                        Verify Email
                      </Button>
                      <Button onClick={handleSendVerificationCode} loading={sendingCode}>
                        Resend Code
                      </Button>
                    </Space>
                  </Space>
                </div>
              )}
            </>
          )}
        </div>
      </Card>

      {/* Delete Account */}
      <Card className="security-card danger-card" title="Delete Account">
        <Alert
          type="error"
          message="This action is permanent and cannot be undone. All your data will be deleted."
          style={{ marginBottom: 16 }}
        />
        <p className="danger-text">
          Deleting your account will permanently remove all your notes, recordings, and settings.
        </p>
        <Button
          type="primary"
          danger
          icon={<DeleteOutlined />}
          onClick={() => setDeleteModalVisible(true)}
        >
          Delete Account
        </Button>
      </Card>

      {/* Delete Confirmation Modal */}
      <Modal
        open={deleteModalVisible}
        title="Confirm Account Deletion"
        onCancel={() => {
          setDeleteModalVisible(false);
          setDeleteConfirmText('');
        }}
        footer={null}
        className="delete-modal"
      >
        <Alert
          type="error"
          message="Warning: This action cannot be undone!"
          style={{ marginBottom: 16 }}
        />
        <p>
          Are you absolutely sure you want to delete your account? This will permanently remove:
        </p>
        <ul className="delete-list">
          <li>All your notes and recordings</li>
          <li>All your settings and preferences</li>
          <li>Your subscription (no refunds will be issued)</li>
          <li>All your device pairings</li>
        </ul>
        <p className="confirm-instruction">
          Type <strong>DELETE</strong> to confirm:
        </p>
        <Input
          placeholder="Type DELETE"
          value={deleteConfirmText}
          onChange={(e) => setDeleteConfirmText(e.target.value)}
          style={{ marginBottom: 16 }}
        />
        <Space style={{ width: '100%', justifyContent: 'flex-end' }}>
          <Button
            onClick={() => {
              setDeleteModalVisible(false);
              setDeleteConfirmText('');
            }}
          >
            Cancel
          </Button>
          <Button
            type="primary"
            danger
            onClick={handleDeleteAccount}
            loading={deleting}
            disabled={deleteConfirmText !== 'DELETE'}
          >
            Delete My Account
          </Button>
        </Space>
      </Modal>
    </div>
  );
}
