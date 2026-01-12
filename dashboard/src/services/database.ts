import { useApi } from '@/composables/useApi'
import type { TableData, TableRow, Filter } from '@/types'

const { api } = useApi()

export const databaseService = {
    /**
     * Get list of all tables in the database
     */
    async getTables(): Promise<string[]> {
        const response = await api.get('/api/tables')
        return response.data
    },

    /**
     * Get table schema information
     */
    async getTableSchema(tableName: string): Promise<{
        name: string
        columns: Array<{
            name: string
            type: string
            nullable: boolean
            primary_key: boolean
        }>
    }> {
        const response = await api.get(`/api/tables/${tableName}/schema`)
        return response.data
    },

    /**
     * Get table data with pagination and filtering
     */
    async getTableData(
        tableName: string,
        options?: {
            limit?: number
            offset?: number
            filters?: Filter[]
            orderBy?: string
            orderDir?: 'asc' | 'desc'
        }
    ): Promise<TableData> {
        const params = new URLSearchParams()
        if (options?.limit) params.append('limit', options.limit.toString())
        if (options?.offset) params.append('offset', options.offset.toString())
        if (options?.orderBy) params.append('order_by', options.orderBy)
        if (options?.orderDir) params.append('order_dir', options.orderDir)
        if (options?.filters) {
            params.append('filters', JSON.stringify(options.filters))
        }

        const response = await api.get(`/api/tables/${tableName}/data?${params}`)
        return response.data
    },

    /**
     * Execute a SQL query
     */
    async executeQuery(query: string): Promise<{
        rows: TableRow[]
        columns: string[]
        rowCount: number
        executionTime: number
    }> {
        const response = await api.post('/api/query', { query })
        return response.data
    },

    /**
     * Insert a row into a table
     */
    async insertRow(tableName: string, data: TableRow): Promise<TableRow> {
        const response = await api.post(`/api/tables/${tableName}/rows`, data)
        return response.data
    },

    /**
     * Update a row in a table
     */
    async updateRow(tableName: string, rowId: string | number, data: Partial<TableRow>): Promise<TableRow> {
        const response = await api.patch(`/api/tables/${tableName}/rows/${rowId}`, data)
        return response.data
    },

    /**
     * Delete a row from a table
     */
    async deleteRow(tableName: string, rowId: string | number): Promise<void> {
        await api.delete(`/api/tables/${tableName}/rows/${rowId}`)
    },

    /**
     * Create a new table
     */
    async createTable(schema: {
        name: string
        columns: Array<{
            name: string
            type: string
            nullable?: boolean
            primary_key?: boolean
            default?: unknown
        }>
    }): Promise<void> {
        await api.post('/api/tables', schema)
    },

    /**
     * Drop a table
     */
    async dropTable(tableName: string): Promise<void> {
        await api.delete(`/api/tables/${tableName}`)
    },

    /**
     * Get database statistics
     */
    async getStatistics(): Promise<{
        total_tables: number
        total_rows: number
        database_size: number
        wal_size: number
    }> {
        const response = await api.get('/api/database/stats')
        return response.data
    },
}
