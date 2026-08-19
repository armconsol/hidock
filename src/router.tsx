import { createBrowserRouter } from 'react-router-dom';
import { AppLayout } from './components/Layout/AppLayout';
import { HomePage } from './pages/Home';
import { NotesPage } from './pages/Notes';
import { NoteDetailPage } from './pages/NoteDetail';
import { WhispersPage } from './pages/Whispers';
import { LiveTranslationPage } from './pages/LiveTranslation';
import { TodoPage } from './pages/Todo';
import { DevicesPage } from './pages/Devices';
import { SettingsPage } from './pages/Settings';
import { SubscriptionPage } from './pages/Subscription';
import { ProfilePage } from './pages/Profile';
import { SecurityPage } from './pages/Security';
import { LoginPage } from './pages/Login';
import { SyncDemo } from './pages/SyncDemo';

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/',
    element: <AppLayout />,
    children: [
      {
        path: '/',
        element: <HomePage />,
      },
      {
        path: '/home',
        element: <HomePage />,
      },
      {
        path: '/notes',
        element: <NotesPage />,
      },
      {
        path: '/notes/:id',
        element: <NoteDetailPage />,
      },
      {
        path: '/whispers',
        element: <WhispersPage />,
      },
      {
        path: '/live',
        element: <LiveTranslationPage />,
      },
      {
        path: '/todo',
        element: <TodoPage />,
      },
      {
        path: '/devices',
        element: <DevicesPage />,
      },
      {
        path: '/settings',
        element: <SettingsPage />,
      },
      {
        path: '/subscription',
        element: <SubscriptionPage />,
      },
      {
        path: '/profile',
        element: <ProfilePage />,
      },
      {
        path: '/security',
        element: <SecurityPage />,
      },
      {
        path: '/sync-demo',
        element: <SyncDemo />,
      },
    ],
  },
]);
