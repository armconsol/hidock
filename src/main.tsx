import ReactDOM from "react-dom/client";
import { RouterProvider } from 'react-router-dom';
import { ConfigProvider } from 'antd';
import { ErrorBoundary } from './components/ErrorBoundary';
import { router } from './router';
import 'antd/dist/reset.css';
import './styles/theme.css';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <ErrorBoundary>
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: '#1890ff',
          borderRadius: 6,
        },
      }}
    >
      <RouterProvider router={router} />
    </ConfigProvider>
  </ErrorBoundary>
);
