// Core TypeScript interfaces for AeroDB Admin Dashboard

export interface User {
    id: string
    email: string
    name?: string
    role: string
    created_at: string
    last_login?: string
}

export interface AuthTokens {
    access_token: string
    refresh_token?: string
    expires_in: number
}

export interface ApiError {
    message: string
    code?: string
    details?: unknown
}

export interface TableRow {
    [key: string]: unknown
}

export interface TableData {
    rows: TableRow[]
    columns: string[]
    total: number
    limit: number
    offset: number
}

export interface Filter {
    field: string
    operator: 'eq' | 'gt' | 'lt' | 'gte' | 'lte' | 'like' | 'in'
    value: string | number | boolean
}

export interface Session {
    id: string
    user_id: string
    device?: string
    last_active: string
    expires_at: string
}

export interface Bucket {
    name: string
    public: boolean
    created_at: string
    updated_at: string
}

export interface StorageFile {
    name: string
    size: number
    content_type: string
    created_at: string
    updated_at: string
}

export interface Subscription {
    id: string
    user_id: string
    channel: string
    filter?: Record<string, unknown>
    connected_at: string
}

export interface ClusterNode {
    id: string
    role: 'authority' | 'replica'
    status: 'online' | 'offline'
    replication_lag?: number
}

export interface LogEntry {
    timestamp: string
    level: 'debug' | 'info' | 'warn' | 'error'
    module: string
    message: string
}

export interface MetricDataPoint {
    timestamp: string
    value: number
}
