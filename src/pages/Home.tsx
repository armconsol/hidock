import { CalendarWidget } from '../components/Calendar/CalendarWidget';
import './Home.css';

export function HomePage() {
  return (
    <div className="home-page">
      <h1>Home Dashboard</h1>
      <div className="dashboard-widgets">
        <div className="widget-container">
          <CalendarWidget />
        </div>
        {/* Additional widgets can be added here */}
      </div>
    </div>
  );
}
