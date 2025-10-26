// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path'; // <--- Isso agora funciona por causa do Passo 1

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  // O "Porquê": Diz pro Vite que '@/' aponta para 'src/'
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});