import { useApi } from '@/composables/useApi'
import type { LogEntry, MetricDataPoint, LogFilter } from '@/types'

const { api } = useApi()

export const observabilityService = {
    /**
     * Get logs with filtering
     */
    async getLogs(options?: {
        limit?: number
        offset?: number
        level?: 'debug' | 'info' | 'warn' | 'error'
        module?: string
        search?: string
        since?: string
        until?: string
    }): Promise<{
        logs: LogEntry[]
        total: number
    }> {
        const params = new URLSearchParams()
        if (options?.limit) params.append('limit', options.limit.toString())
        if (options?.offset) params.append('offset', options.offset.toString())
        if (options?.level) params.append('level', options.level)
        if (options?.module) params.append('module', options.module)
        if (options?.search) params.append('search', options.search)
        if (options?.since) params.append('since', options.since)
        if (options?.until) params.append('until', options.until)

        const response = await api.get(`/observability/logs?${params}`)
        return response.data
    },

    /**
     * Stream logs in real-time (returns event source URL)
     */
    getLogStreamUrl(filter?: LogFilter): string {
        const { baseURL } = useApi()
        const params = new URLSearchParams()
        if (filter?.level) params.append('level', filter.level)
        if (filter?.module) params.append('module', filter.module)

        return `${baseURL}/observability/logs/stream?${params}`
    },

    /**
     * Get available log modules
     */
    async getLogModules(): Promise<string[]> {
        const response = await api.get('/observability/logs/modules')
        return response.data
    },

    /**
     * Get metrics data
     */
    async getMetrics(
        metricName: string,
        options?: {
            since?: string
            until?: string
            resolution?: '1m' | '5m' | '15m' | '1h'
        }
    ): Promise<{
        metric: string
        datapoints: MetricDataPoint[]
    }> {
        const params = new URLSearchParams()
        params.append('metric', metricName)
        if (options?.since) params.append('since', options.since)
        if (options?.until) params.append('until', options.until)
        if (options?.resolution) params.append('resolution', options.resolution)

        const response = await api.get(`/observability/metrics?${params}`)
        return response.data
    },

    /**
     * Get multiple metrics at once
     */
    async getMultipleMetrics(
        metricNames: string[],
        options?: {
            since?: string
            until?: string
            resolution?: '1m' | '5m' | '15m' | '1h'
        }
    ): Promise<Record<string, MetricDataPoint[]>> {
        const params = new URLSearchParams()
        metricNames.forEach((name) => params.append('metrics[]', name))
        if (options?.since) params.append('since', options.since)
        if (options?.until) params.append('until', options.until)
        if (options?.resolution) params.append('resolution', options.resolution)

        const response = await api.get(`/observability/metrics/batch?${params}`)
        return response.data
    },

    /**
     * Get available metric names
     */
    async getMetricNames(): Promise<Array<{
        name: string
        description: string
        unit: string
    }>> {
        const response = await api.get('/observability/metrics/available')
        return response.data
    },

    /**
     * Get current system metrics snapshot
     */
    async getCurrentMetrics(): Promise<{
        cpu_usage_percent: number
        memory_usage_bytes: number
        memory_total_bytes: number
        disk_usage_bytes: number
        disk_total_bytes: number
        active_connections: number
        queries_per_second: number
        wal_size_bytes: number
    }> {
        const response = await api.get('/observability/metrics/current')
        return response.data
    },

    /**
     * Get performance statistics
     */
    async getPerformanceStats(): Promise<{
        avg_query_duration_ms: number
        p95_query_duration_ms: number
        p99_query_duration_ms: number
        total_queries: number
        cache_hit_rate: number
        index_efficiency: number
    }> {
        const response = await api.get('/observability/performance')
        return response.data
    },

    /**
     * Export logs to file
     */
    async exportLogs(options?: {
        format?: 'json' | 'csv'
        since?: string
        until?: string
        level?: 'debug' | 'info' | 'warn' | 'error'
    }): Promise<Blob> {
        const params = new URLSearchParams()
        if (options?.format) params.append('format', options.format)
        if (options?.since) params.append('since', options.since)
        if (options?.until) params.append('until', options.until)
        if (options?.level) params.append('level', options.level)

        const response = await api.get(`/observability/logs/export?${params}`, {
            responseType: 'blob',
        })
        return response.data
    },
}
