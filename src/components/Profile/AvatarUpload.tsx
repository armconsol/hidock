import { useState, useRef } from 'react';
import { Button, Avatar, message, Spin, Modal } from 'antd';
import { CameraOutlined, UserOutlined } from '@ant-design/icons';
import './AvatarUpload.css';

interface AvatarUploadProps {
  currentAvatar?: string;
  userName?: string;
  onUpload: (file: File) => Promise<string>;
}

export function AvatarUpload({ currentAvatar, userName, onUpload }: AvatarUploadProps) {
  const [uploading, setUploading] = useState(false);
  const [previewUrl, setPreviewUrl] = useState<string | undefined>(currentAvatar);
  const [showPreview, setShowPreview] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = async (file: File) => {
    // Validate file type
    if (!file.type.startsWith('image/')) {
      message.error('Please select an image file');
      return false;
    }

    // Validate file size (5MB max)
    if (file.size > 5 * 1024 * 1024) {
      message.error('Image size must be less than 5MB');
      return false;
    }

    // Show preview
    const reader = new FileReader();
    reader.onload = (e) => {
      setPreviewUrl(e.target?.result as string);
    };
    reader.readAsDataURL(file);

    // Upload
    setUploading(true);
    try {
      const avatarUrl = await onUpload(file);
      setPreviewUrl(avatarUrl);
      message.success('Avatar updated successfully');
    } catch (error) {
      message.error('Failed to upload avatar');
      console.error('Avatar upload error:', error);
      // Revert preview on error
      setPreviewUrl(currentAvatar);
    } finally {
      setUploading(false);
    }

    return false; // Prevent default upload behavior
  };

  const handleButtonClick = () => {
    fileInputRef.current?.click();
  };

  const handlePreviewClick = () => {
    if (previewUrl) {
      setShowPreview(true);
    }
  };

  return (
    <div className="avatar-upload">
      <div className="avatar-container">
        <div className="avatar-wrapper" onClick={handlePreviewClick} role="button" tabIndex={0}>
          {previewUrl ? (
            <Avatar size={120} className="avatar-image">
              <img src={previewUrl} alt={userName || 'User avatar'} />
            </Avatar>
          ) : (
            <Avatar size={120} className="avatar-placeholder" icon={<UserOutlined />} />
          )}
          {uploading && (
            <div className="avatar-loading-overlay">
              <Spin />
            </div>
          )}
        </div>
        <input
          ref={fileInputRef}
          type="file"
          accept="image/*"
          style={{ display: 'none' }}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) {
              handleFileSelect(file);
            }
          }}
        />
      </div>

      <Button
        type="default"
        icon={<CameraOutlined />}
        onClick={handleButtonClick}
        loading={uploading}
        disabled={uploading}
      >
        {previewUrl ? 'Change Avatar' : 'Upload Avatar'}
      </Button>

      <p className="avatar-hint">
        JPG, PNG, or GIF. Max size 5MB.
      </p>

      <Modal
        open={showPreview}
        footer={null}
        onCancel={() => setShowPreview(false)}
        style={{ width: 'auto', maxWidth: '90vw' }}
      >
        {previewUrl && (
          <img
            src={previewUrl}
            alt={userName || 'User avatar'}
            style={{ width: '100%', height: 'auto', maxHeight: '80vh' }}
          />
        )}
      </Modal>
    </div>
  );
}
