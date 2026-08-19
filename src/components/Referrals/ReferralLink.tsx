import { useState } from 'react';
import { Card, Input, Button, Space, Message, Tooltip } from '@arco-design/web-react';
import { IconCopy, IconShareAlt, IconQrcode } from '@arco-design/web-react/icon';
import './ReferralLink.css';

interface ReferralLinkProps {
  referralCode: string;
  referralLink: string;
  onShare?: (platform: 'twitter' | 'facebook' | 'whatsapp' | 'email') => void;
}

export function ReferralLink({ referralCode, referralLink, onShare }: ReferralLinkProps) {
  const [showQR, setShowQR] = useState(false);

  const handleCopyCode = () => {
    navigator.clipboard.writeText(referralCode);
    Message.success('Referral code copied to clipboard!');
  };

  const handleCopyLink = () => {
    navigator.clipboard.writeText(referralLink);
    Message.success('Referral link copied to clipboard!');
  };

  const handleShare = (platform: 'twitter' | 'facebook' | 'whatsapp' | 'email') => {
    const message = encodeURIComponent(
      `Join HiNotes using my referral code: ${referralCode}\n${referralLink}`
    );

    let shareUrl = '';
    switch (platform) {
      case 'twitter':
        shareUrl = `https://twitter.com/intent/tweet?text=${message}`;
        break;
      case 'facebook':
        shareUrl = `https://www.facebook.com/sharer/sharer.php?u=${encodeURIComponent(referralLink)}`;
        break;
      case 'whatsapp':
        shareUrl = `https://wa.me/?text=${message}`;
        break;
      case 'email':
        shareUrl = `mailto:?subject=${encodeURIComponent('Join HiNotes')}&body=${message}`;
        break;
    }

    window.open(shareUrl, '_blank');
    onShare?.(platform);
  };

  const generateQRCodeUrl = () => {
    // Using a QR code API service
    return `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(referralLink)}`;
  };

  return (
    <Card className="referral-link-card" title="Your Referral Link">
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        {/* Referral Code */}
        <div className="referral-code-section">
          <label className="referral-label">Referral Code</label>
          <Input.Group compact>
            <Input
              style={{ width: 'calc(100% - 100px)' }}
              value={referralCode}
              readOnly
              size="large"
            />
            <Tooltip content="Copy code">
              <Button
                icon={<IconCopy />}
                onClick={handleCopyCode}
                size="large"
                type="primary"
              >
                Copy
              </Button>
            </Tooltip>
          </Input.Group>
        </div>

        {/* Referral Link */}
        <div className="referral-link-section">
          <label className="referral-label">Referral Link</label>
          <Input.Group compact>
            <Input
              style={{ width: 'calc(100% - 100px)' }}
              value={referralLink}
              readOnly
            />
            <Tooltip content="Copy link">
              <Button
                icon={<IconCopy />}
                onClick={handleCopyLink}
                type="outline"
              >
                Copy
              </Button>
            </Tooltip>
          </Input.Group>
        </div>

        {/* Share Buttons */}
        <div className="share-section">
          <label className="referral-label">Share via</label>
          <Space size="medium" wrap>
            <Button
              icon={<IconShareAlt />}
              onClick={() => handleShare('twitter')}
              className="share-button twitter"
            >
              Twitter
            </Button>
            <Button
              icon={<IconShareAlt />}
              onClick={() => handleShare('facebook')}
              className="share-button facebook"
            >
              Facebook
            </Button>
            <Button
              icon={<IconShareAlt />}
              onClick={() => handleShare('whatsapp')}
              className="share-button whatsapp"
            >
              WhatsApp
            </Button>
            <Button
              icon={<IconShareAlt />}
              onClick={() => handleShare('email')}
              className="share-button email"
            >
              Email
            </Button>
          </Space>
        </div>

        {/* QR Code */}
        <div className="qr-code-section">
          <Button
            icon={<IconQrcode />}
            onClick={() => setShowQR(!showQR)}
            type="outline"
            long
          >
            {showQR ? 'Hide QR Code' : 'Show QR Code'}
          </Button>
          {showQR && (
            <div className="qr-code-display">
              <img
                src={generateQRCodeUrl()}
                alt="Referral QR Code"
                className="qr-code-image"
              />
              <p className="qr-code-hint">Scan to share your referral link</p>
            </div>
          )}
        </div>
      </Space>
    </Card>
  );
}
