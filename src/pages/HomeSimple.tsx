export function HomeSimple() {
  return (
    <div style={{ padding: '24px', maxWidth: '1200px', margin: '0 auto' }}>
      <h1 style={{ fontSize: '24px', marginBottom: '24px' }}>Home Dashboard</h1>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '20px' }}>
        {/* Recent Notes Widget */}
        <div style={{
          padding: '20px',
          border: '1px solid #e5e6eb',
          borderRadius: '4px',
          background: 'white'
        }}>
          <h2 style={{ fontSize: '18px', marginBottom: '16px' }}>Recent Notes</h2>
          <p style={{ color: '#86909c' }}>Your recent notes will appear here</p>
          <div style={{ marginTop: '12px', fontSize: '14px', color: '#4e5969' }}>
            <p>• Create notes via the Notes tab</p>
            <p>• Record with HiDoc P1 device</p>
            <p>• Import audio files</p>
          </div>
        </div>

        {/* Calendar Widget */}
        <div style={{
          padding: '20px',
          border: '1px solid #e5e6eb',
          borderRadius: '4px',
          background: 'white'
        }}>
          <h2 style={{ fontSize: '18px', marginBottom: '16px' }}>Today's Schedule</h2>
          <p style={{ color: '#86909c' }}>Calendar integration available</p>
          <div style={{ marginTop: '12px', fontSize: '14px', color: '#4e5969' }}>
            <p>• Connect Google Calendar</p>
            <p>• Sync events automatically</p>
            <p>• Create notes from meetings</p>
          </div>
        </div>

        {/* To-Do Widget */}
        <div style={{
          padding: '20px',
          border: '1px solid #e5e6eb',
          borderRadius: '4px',
          background: 'white'
        }}>
          <h2 style={{ fontSize: '18px', marginBottom: '16px' }}>To-Dos</h2>
          <p style={{ color: '#86909c' }}>Your tasks will appear here</p>
          <div style={{ marginTop: '12px', fontSize: '14px', color: '#4e5969' }}>
            <p>• Create tasks manually</p>
            <p>• Extract from notes</p>
            <p>• Convert whispers to tasks</p>
          </div>
        </div>
      </div>

      <div style={{
        marginTop: '24px',
        padding: '20px',
        background: '#f7f8fa',
        borderRadius: '4px',
        border: '1px solid #e5e6eb'
      }}>
        <h3 style={{ fontSize: '16px', marginBottom: '12px' }}>Getting Started</h3>
        <ul style={{ paddingLeft: '20px', lineHeight: '1.8', color: '#4e5969' }}>
          <li>Connect your HiDoc P1 USB device to import recordings</li>
          <li>Configure OAuth credentials for Google/Apple Sign-In (Settings → OAuth Configuration)</li>
          <li>Create your first note or whisper using the sidebar navigation</li>
          <li>Enable live translation for real-time transcription</li>
        </ul>
      </div>
    </div>
  );
}
