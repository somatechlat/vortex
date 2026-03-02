export interface VortexConfig {
    coreBaseUrl: string;
    adminBaseUrl: string;
    wsUrl: string;
}

export class VortexClient {
    private config: VortexConfig;

    constructor(config: VortexConfig) {
        this.config = config;
    }

    async fetchGraphs() {
        const res = await fetch(`${this.config.coreBaseUrl}/api/graph`);
        if (!res.ok) throw new Error('Failed to fetch graphs');
        return res.json();
    }

    async executeGraph(graphId: string) {
        const res = await fetch(`${this.config.coreBaseUrl}/api/graph/${graphId}/execute`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ priority: 'high' })
        });
        return res.json();
    }

    // MCP Methods
    async listMcpNodes() {
        const res = await fetch(`${this.config.coreBaseUrl}/api/nodes/mcp`);
        return res.json();
    }

    // Admin/Monitoring Methods
    async getRunHistory() {
        const res = await fetch(`${this.config.adminBaseUrl}/api/runs`);
        return res.json();
    }
}

// @ts-ignore
const env = import.meta.env || {};

// Singleton Instance
export const vortexClient = new VortexClient({
    coreBaseUrl: env.VITE_CORE_API_URL || 'http://localhost:11188',
    adminBaseUrl: env.VITE_ADMIN_API_URL || 'http://localhost:8000/api',
    wsUrl: env.VITE_WS_URL || 'ws://localhost:11188/ws'
});
