import { useState, useEffect } from 'react';
import { Card, Form, Input, Select, Button, message, Spin, Divider } from 'antd';
import { EditOutlined, SaveOutlined, CloseOutlined } from '@ant-design/icons';
import { AvatarUpload } from '../components/Profile/AvatarUpload';
import { useAuthStore } from '../store/authStore';
import { UserProfile } from '../types/user';
import './Profile.css';

const FormItem = Form.Item;
const { Option } = Select;

export function ProfilePage() {
  const [form] = Form.useForm();
  const { user, setUser } = useAuthStore();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [editing, setEditing] = useState(false);
  const [profile, setProfile] = useState<UserProfile | null>(null);

  useEffect(() => {
    loadProfile();
  }, []);

  const loadProfile = async () => {
    setLoading(true);
    try {
      // TODO: Replace with actual API call
      // API endpoint: POST /v1/user/info

      const mockProfile: UserProfile = {
        id: user?.id || 'user123',
        email: user?.email || 'user@example.com',
        name: user?.name || 'User Name',
        avatar: user?.avatar,
        region: 'us',
        emailVerified: true,
        createdAt: new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString(),
        updatedAt: new Date().toISOString(),
      };

      setProfile(mockProfile);
      form.setFieldsValue({
        name: mockProfile.name,
        email: mockProfile.email,
        region: mockProfile.region,
      });
    } catch (error) {
      message.error('Failed to load profile');
      console.error('Error loading profile:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      await form.validateFields();
      const values = form.getFieldsValue();

      setSaving(true);
      try {
        // TODO: Replace with actual API call
        // API endpoint: POST /v1/user/rename (for name)
        // For region, might need additional endpoint or combined update

        const updatedProfile: UserProfile = {
          ...profile!,
          name: values.name,
          region: values.region,
          updatedAt: new Date().toISOString(),
        };

        setProfile(updatedProfile);

        // Update auth store
        if (user) {
          setUser({
            ...user,
            name: values.name,
          });
        }

        message.success('Profile updated successfully');
        setEditing(false);
      } catch (error) {
        message.error('Failed to update profile');
        console.error('Error updating profile:', error);
      } finally {
        setSaving(false);
      }
    } catch (error) {
      // Form validation error
      console.error('Form validation error:', error);
    }
  };

  const handleCancel = () => {
    form.setFieldsValue({
      name: profile?.name,
      region: profile?.region,
    });
    setEditing(false);
  };

  const handleAvatarUpload = async (file: File): Promise<string> => {
    // TODO: Replace with actual API call
    // API endpoint: POST /v1/user/avatar/upload

    // Simulate upload delay
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Create local preview URL
    const previewUrl = URL.createObjectURL(file);

    // Update profile and auth store
    if (profile) {
      const updatedProfile = { ...profile, avatar: previewUrl };
      setProfile(updatedProfile);

      if (user) {
        setUser({ ...user, avatar: previewUrl });
      }
    }

    return previewUrl;
  };

  if (loading) {
    return (
      <div className="profile-page loading">
        <Spin size="large" />
      </div>
    );
  }

  if (!profile) {
    return (
      <div className="profile-page">
        <h1>Profile</h1>
        <Card>
          <p>Failed to load profile. Please try again later.</p>
        </Card>
      </div>
    );
  }

  return (
    <div className="profile-page">
      <h1>Profile</h1>

      <Card className="profile-card">
        <div className="profile-avatar-section">
          <AvatarUpload
            currentAvatar={profile.avatar}
            userName={profile.name}
            onUpload={handleAvatarUpload}
          />
        </div>

        <Divider />

        <Form
          form={form}
          layout="vertical"
          className="profile-form"
          disabled={!editing}
        >
          <FormItem
            label="Name"
            name="name"
            rules={[
              { required: true, message: 'Name is required' },
              { min: 2, message: 'Name must be at least 2 characters' },
              { max: 50, message: 'Name must be less than 50 characters' },
            ]}
          >
            <Input placeholder="Enter your name" />
          </FormItem>

          <FormItem label="Email" name="email">
            <Input
              placeholder="Email"
              disabled
              suffix={
                profile.emailVerified ? (
                  <span className="verified-badge">Verified</span>
                ) : (
                  <span className="unverified-badge">Not Verified</span>
                )
              }
            />
          </FormItem>

          <FormItem
            label="Region"
            name="region"
            rules={[{ required: true, message: 'Region is required' }]}
          >
            <Select placeholder="Select region">
              <Option value="us">United States</Option>
              <Option value="eu">Europe</Option>
              <Option value="asia">Asia</Option>
              <Option value="other">Other</Option>
            </Select>
          </FormItem>

          <div className="profile-actions">
            {editing ? (
              <>
                <Button
                  type="primary"
                  icon={<SaveOutlined />}
                  onClick={handleSave}
                  loading={saving}
                >
                  Save Changes
                </Button>
                <Button
                  icon={<CloseOutlined />}
                  onClick={handleCancel}
                  disabled={saving}
                >
                  Cancel
                </Button>
              </>
            ) : (
              <Button
                type="primary"
                icon={<EditOutlined />}
                onClick={() => setEditing(true)}
              >
                Edit Profile
              </Button>
            )}
          </div>
        </Form>

        <Divider />

        <div className="profile-meta">
          <div className="meta-item">
            <label>Member Since</label>
            <span>
              {new Date(profile.createdAt).toLocaleDateString('en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
              })}
            </span>
          </div>
          <div className="meta-item">
            <label>Last Updated</label>
            <span>
              {new Date(profile.updatedAt).toLocaleDateString('en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
              })}
            </span>
          </div>
        </div>
      </Card>
    </div>
  );
}
