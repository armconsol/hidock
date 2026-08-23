import { Layout, Menu } from 'antd';
import {
  HomeOutlined,
  FileOutlined,
  TranslationOutlined,
  MessageOutlined,
  CheckSquareOutlined,
  SettingOutlined,
  UserOutlined,
  SafetyOutlined,
} from '@ant-design/icons';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { ThemeProvider } from '../ThemeProvider';
import { useAuthLifecycle } from '../../hooks/useAuthLifecycle';
import './AppLayout.css';

const { Sider, Content } = Layout;

export function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();

  // Initialize auth lifecycle management
  useAuthLifecycle();

  const menuItems = [
    {
      key: '/home',
      icon: <HomeOutlined />,
      label: 'Home',
    },
    {
      key: '/notes',
      icon: <FileOutlined />,
      label: 'Notes',
    },
    {
      key: '/live',
      icon: <TranslationOutlined />,
      label: 'Translate',
    },
    {
      key: '/whispers',
      icon: <MessageOutlined />,
      label: 'Whispers',
    },
    {
      key: '/todo',
      icon: <CheckSquareOutlined />,
      label: 'To-Do',
    },
  ];

  const handleMenuClick = (key: string) => {
    navigate(key);
  };

  // Determine selected key from current location
  const selectedKey = location.pathname === '/' ? '/home' : location.pathname;

  return (
    <ThemeProvider>
      <Layout className="app-layout">
        <Sider
          className="app-sidebar"
          width={80}
          collapsible={false}
          trigger={null}
        >
          <div className="sidebar-content">
            <Menu
              className="sidebar-menu"
              selectedKeys={[selectedKey]}
              onClick={({ key }) => handleMenuClick(key)}
              style={{
                width: '100%',
                height: '100%',
              }}
              items={menuItems.map((item) => ({
                key: item.key,
                icon: item.icon,
                label: (
                  <div className="menu-item-content">
                    <span className="menu-item-label">{item.label}</span>
                  </div>
                ),
                className: 'sidebar-menu-item',
              }))}
            />

            <div className="sidebar-footer">
              <div
                className="footer-button"
                onClick={() => navigate('/profile')}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    navigate('/profile');
                  }
                }}
              >
                <UserOutlined />
                <span className="menu-item-label">Profile</span>
              </div>
              <div
                className="footer-button"
                onClick={() => navigate('/security')}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    navigate('/security');
                  }
                }}
              >
                <SafetyOutlined />
                <span className="menu-item-label">Security</span>
              </div>
              <div
                className="footer-button"
                onClick={() => navigate('/settings')}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    navigate('/settings');
                  }
                }}
              >
                <SettingOutlined />
                <span className="menu-item-label">Settings</span>
              </div>
            </div>
          </div>
        </Sider>
        <Content className="app-content">
          <Outlet />
        </Content>
      </Layout>
    </ThemeProvider>
  );
}
