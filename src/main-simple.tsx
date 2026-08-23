import React from "react";
import ReactDOM from "react-dom/client";

function SimpleApp() {
  return (
    <div style={{ padding: '20px', fontFamily: 'sans-serif' }}>
      <h1>HiNotes Desktop - Simple Test</h1>
      <p>If you see this, React is working!</p>
      <p>Next step: Add router...</p>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SimpleApp />
  </React.StrictMode>,
);
