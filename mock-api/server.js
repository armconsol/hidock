import express from 'express';
import cors from 'cors';

const app = express();
const PORT = 3001;

app.use(cors());
app.use(express.json());

// Mock authentication
app.post('/v1/user/signin', (req, res) => {
  const { email, password } = req.body;

  // Simple mock authentication
  if (email && password) {
    res.json({
      token: `mock-token-${Date.now()}`,
      user: {
        id: `user_${Date.now()}`,
        email: email,
        name: 'Test User'
      }
    });
  } else {
    res.status(401).json({ error: 'Invalid credentials' });
  }
});

app.listen(PORT, () => {
  console.log(`Mock HiNotes API running on http://localhost:${PORT}`);
});
