// User types
export interface User {
    id: string;
    email: string;
    created_at: string;
    last_login?: string;
    role?: string;
}

// Session types
export interface Session {
    access_token: string;
    refresh_token?: string;
    expires_at?: number;
    user: User;
}

// Auth types
export interface AuthResponse {
    data: { user: User; session: Session } | null;
    error: ApiError | null;
}

// API types
export interface ApiError {
    message: string;
    status?: number;
    code?: string;
}

export interface ApiResponse<T> {
    data: T | null;
    error: ApiError | null;
}

export interface PaginatedResponse<T> {
    data: T[];
    total: number;
    limit: number;
    offset: number;
    fetched_at: string;
}

// Database types
export interface Collection {
    name: string;
    count: number;
    columns: Column[];
}

export interface Column {
    name: string;
    type: string;
    nullable: boolean;
    default_value?: string;
}

export interface QueryResult {
    rows: Record<string, unknown>[];
    columns: string[];
    execution_time_ms: number;
    affected_rows?: number;
}

// Storage types
export interface Bucket {
    id: string;
    name: string;
    public: boolean;
    file_count: number;
    total_size: number;
    created_at: string;
}

export interface StorageObject {
    id: string;
    bucket_id: string;
    path: string;
    size: number;
    content_type: string;
    created_at: string;
    updated_at: string;
}

// Realtime types
export interface Subscription {
    id: string;
    channel: string;
    user_id: string;
    filter?: string;
    connected_at: string;
}

export interface RealtimeEvent {
    type: "INSERT" | "UPDATE" | "DELETE";
    collection: string;
    old?: Record<string, unknown>;
    new?: Record<string, unknown>;
    timestamp: string;
}

// Observability types
export interface LogEntry {
    id: string;
    timestamp: string;
    level: "ERROR" | "WARN" | "INFO" | "DEBUG";
    module: string;
    message: string;
    metadata?: Record<string, unknown>;
}

export interface Metric {
    name: string;
    value: number;
    timestamp: string;
    labels?: Record<string, string>;
}

// Cluster types
export interface ClusterNode {
    id: string;
    role: "authority" | "replica";
    host: string;
    port: number;
    status: "healthy" | "degraded" | "offline";
    lag_ms?: number;
    last_heartbeat: string;
}

// Filter types
export interface Filter {
    field: string;
    operator: FilterOperator;
    value: string | number | boolean;
}

export type FilterOperator =
    | "eq"
    | "neq"
    | "gt"
    | "gte"
    | "lt"
    | "lte"
    | "like"
    | "ilike"
    | "in";

// Table state
export interface TableState {
    collection: string;
    filters: Filter[];
    orderBy?: { field: string; ascending: boolean };
    limit: number;
    offset: number;
}
