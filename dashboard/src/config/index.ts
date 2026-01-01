// Environment-based configuration
export const config = {
    // API URLs
    apiUrl: import.meta.env.VITE_AERODB_URL || "http://localhost:54321",
    wsUrl:
        import.meta.env.VITE_WS_URL || "ws://localhost:54321/realtime/v1",

    // API Endpoints
    endpoints: {
        rest: "/rest/v1",
        auth: "/auth",
        storage: "/storage/v1",
        realtime: "/realtime/v1",
        control: "/control",
        observability: "/observability",
    },

    // Pagination defaults
    pagination: {
        defaultLimit: 20,
        maxLimit: 100,
    },

    // Timeouts
    timeouts: {
        query: 30000, // 30 seconds
        upload: 300000, // 5 minutes
        default: 10000, // 10 seconds
    },

    // Stale data threshold (10 minutes)
    staleDataThreshold: 10 * 60 * 1000,
} as const;

export type Config = typeof config;
