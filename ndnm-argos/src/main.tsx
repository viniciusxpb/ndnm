import React from 'react';
import ReactDOM from 'react-dom/client';
import App from '@/App';
import '@/style.scss';
// Importa o CSS base do React Flow aqui!
import '@xyflow/react/dist/style.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);