import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [sveltekit()],
    server: {
        port: 11100,
        host: true
    },
    build: {
        target: 'esnext'
    }
});
