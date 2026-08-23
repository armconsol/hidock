import ReactDOM from "react-dom/client";

// Ultra-simple test - no router, no Ant Design, just React
function DebugApp() {
  console.log("DebugApp rendering");
  return (
    <div style={{ padding: '40px', fontSize: '24px', fontFamily: 'sans-serif' }}>
      <h1>Debug Test</h1>
      <p>If you see this, React is working.</p>
      <p>Current time: {new Date().toLocaleTimeString()}</p>
    </div>
  );
}

const root = document.getElementById("root");
console.log("Root element:", root);

if (root) {
  ReactDOM.createRoot(root as HTMLElement).render(<DebugApp />);
} else {
  console.error("Root element not found!");
}
