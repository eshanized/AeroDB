import { useApi } from '@/composables/useApi'
import type { Function, FunctionLog, FunctionInvocation } from '@/types'

const { api } = useApi()

export const functionsService = {
    /**
     * Get all functions
     */
    async getFunctions(): Promise<Function[]> {
        const response = await api.get('/functions')
        return response.data
    },

    /**
     * Get a specific function
     */
    async getFunction(functionId: string): Promise<Function> {
        const response = await api.get(`/functions/${functionId}`)
        return response.data
    },

    /**
     * Create a new function
     */
    async createFunction(data: {
        name: string
        runtime: 'deno' | 'wasm'
        code: string
        env_vars?: Record<string, string>
        triggers?: Array<{
            type: 'http' | 'cron' | 'event'
            config: Record<string, unknown>
        }>
    }): Promise<Function> {
        const response = await api.post('/functions', data)
        return response.data
    },

    /**
     * Update a function
     */
    async updateFunction(functionId: string, data: {
        code?: string
        env_vars?: Record<string, string>
        triggers?: Array<{
            type: 'http' | 'cron' | 'event'
            config: Record<string, unknown>
        }>
        enabled?: boolean
    }): Promise<Function> {
        const response = await api.patch(`/functions/${functionId}`, data)
        return response.data
    },

    /**
     * Delete a function
     */
    async deleteFunction(functionId: string): Promise<void> {
        await api.delete(`/functions/${functionId}`)
    },

    /**
     * Invoke a function
     */
    async invokeFunction(
        functionId: string,
        payload?: Record<string, unknown>
    ): Promise<FunctionInvocation> {
        const response = await api.post(`/functions/${functionId}/invoke`, payload || {})
        return response.data
    },

    /**
     * Get function logs
     */
    async getFunctionLogs(
        functionId: string,
        options?: {
            limit?: number
            offset?: number
            level?: 'debug' | 'info' | 'warn' | 'error'
            since?: string
        }
    ): Promise<{
        logs: FunctionLog[]
        total: number
    }> {
        const params = new URLSearchParams()
        if (options?.limit) params.append('limit', options.limit.toString())
        if (options?.offset) params.append('offset', options.offset.toString())
        if (options?.level) params.append('level', options.level)
        if (options?.since) params.append('since', options.since)

        const response = await api.get(`/functions/${functionId}/logs?${params}`)
        return response.data
    },

    /**
     * Get function invocation history
     */
    async getInvocations(
        functionId: string,
        options?: {
            limit?: number
            offset?: number
        }
    ): Promise<{
        invocations: FunctionInvocation[]
        total: number
    }> {
        const params = new URLSearchParams()
        if (options?.limit) params.append('limit', options.limit.toString())
        if (options?.offset) params.append('offset', options.offset.toString())

        const response = await api.get(`/functions/${functionId}/invocations?${params}`)
        return response.data
    },

    /**
     * Get function statistics
     */
    async getFunctionStats(functionId: string): Promise<{
        total_invocations: number
        success_count: number
        error_count: number
        avg_duration_ms: number
        last_invoked_at?: string
    }> {
        const response = await api.get(`/functions/${functionId}/stats`)
        return response.data
    },
}
