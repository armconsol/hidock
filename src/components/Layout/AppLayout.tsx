import { Outlet } from 'react-router-dom';

export function AppLayout() {
  return (
    <div className="app-layout">
      <aside className="sidebar">
        <nav>{/* Sidebar navigation will go here */}</nav>
      </aside>
      <main className="main-content">
        <Outlet />
      </main>
    </div>
  );
}
