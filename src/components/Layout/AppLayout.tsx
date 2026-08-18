import { Layout, Menu } from '@arco-design/web-react';
import {
  IconHome,
  IconFile,
  IconLanguage,
  IconMessage,
  IconCheckSquare,
  IconSettings,
} from '@arco-design/web-react/icon';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { ThemeProvider } from '../ThemeProvider';
import './AppLayout.css';

const MenuItem = Menu.Item;
const Sider = Layout.Sider;
const Content = Layout.Content;

export function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();

  const menuItems = [
    {
      key: '/home',
      icon: <IconHome />,
      label: 'Home',
    },
    {
      key: '/notes',
      icon: <IconFile />,
      label: 'Notes',
    },
    {
      key: '/live',
      icon: <IconLanguage />,
      label: 'Translate',
    },
    {
      key: '/whispers',
      icon: <IconMessage />,
      label: 'Whispers',
    },
    {
      key: '/todo',
      icon: <IconCheckSquare />,
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
              onClickMenuItem={handleMenuClick}
              style={{
                width: '100%',
                height: '100%',
              }}
            >
              {menuItems.map((item) => (
                <MenuItem key={item.key} className="sidebar-menu-item">
                  <div className="menu-item-content">
                    {item.icon}
                    <span className="menu-item-label">{item.label}</span>
                  </div>
                </MenuItem>
              ))}
            </Menu>

            <div className="sidebar-footer">
              <div
                className="settings-button"
                onClick={() => navigate('/settings')}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    navigate('/settings');
                  }
                }}
              >
                <IconSettings />
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
