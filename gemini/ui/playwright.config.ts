import { defineConfig } from '@playwright/test';

export default defineConfig({
    testDir: 'tests',
    use: {
        baseURL: 'http://localhost:11100',
    },
    reporter: 'list',
});
