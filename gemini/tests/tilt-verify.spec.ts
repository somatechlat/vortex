import { test, expect } from '@playwright/test';

test.describe('Tilt Dashboard (Port 1350)', () => {
    test('Tilt dashboard loads successfully', async ({ page }) => {
        const response = await page.goto('http://localhost:1350/');
        expect(response?.status()).toBe(200);

        // Verify Tilt UI loads
        await expect(page).toHaveTitle(/Tilt/);
        console.log('✅ Tilt dashboard accessible at http://localhost:1350');
    });

    test('Tilt API returns resource list', async ({ request }) => {
        const response = await request.get('http://localhost:1350/api/view');
        expect(response.status()).toBe(200);

        const data = await response.json();
        expect(data).toHaveProperty('logList');
        console.log('✅ Tilt API responding with resource data');
    });

    test('Verify VORTEX resources registered', async ({ request }) => {
        const response = await request.get('http://localhost:1350/api/view');
        const data = await response.json();

        const spans = data.logList?.spans || {};
        const manifests = Object.values(spans)
            .map((s: any) => s.manifestName)
            .filter(Boolean);

        console.log('📦 Registered manifests:', manifests);
        expect(manifests.length).toBeGreaterThan(0);
    });
});
