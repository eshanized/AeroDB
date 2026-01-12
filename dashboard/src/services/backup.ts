import { useApi } from '@/composables/useApi'
import type { BackupJob, BackupManifest, RestoreJob } from '@/types'

const { api } = useApi()

export const backupService = {
    /**
     * Create a new backup
     */
    async createBackup(options?: {
        incremental?: boolean
        compression?: 'none' | 'gzip' | 'zstd'
    }): Promise<BackupJob> {
        const response = await api.post('/backup/create', options || {})
        return response.data
    },

    /**
     * List all backups
     */
    async listBackups(): Promise<Array<{
        id: string
        manifest: BackupManifest
        created_at: string
        size_bytes: number
    }>> {
        const response = await api.get('/backup/list')
        return response.data
    },

    /**
     * Get backup details
     */
    async getBackup(backupId: string): Promise<{
        id: string
        manifest: BackupManifest
        created_at: string
        size_bytes: number
        files: string[]
    }> {
        const response = await api.get(`/backup/${backupId}`)
        return response.data
    },

    /**
     * Delete a backup
     */
    async deleteBackup(backupId: string): Promise<void> {
        await api.delete(`/backup/${backupId}`)
    },

    /**
     * Download a backup
     */
    async downloadBackup(backupId: string): Promise<Blob> {
        const response = await api.get(`/backup/${backupId}/download`, {
            responseType: 'blob',
        })
        return response.data
    },

    /**
     * Get backup job status
     */
    async getBackupStatus(jobId: string): Promise<BackupJob> {
        const response = await api.get(`/backup/jobs/${jobId}`)
        return response.data
    },

    /**
     * Restore from a backup
     */
    async restoreBackup(backupId: string, options?: {
        validate_only?: boolean
        target_path?: string
    }): Promise<RestoreJob> {
        const response = await api.post(`/backup/${backupId}/restore`, options || {})
        return response.data
    },

    /**
     * Upload a backup file for restore
     */
    async uploadBackup(
        file: File,
        onProgress?: (progress: number) => void
    ): Promise<{ backup_id: string }> {
        const formData = new FormData()
        formData.append('backup', file)

        const response = await api.post('/backup/upload', formData, {
            headers: {
                'Content-Type': 'multipart/form-data',
            },
            onUploadProgress: (progressEvent) => {
                if (onProgress && progressEvent.total) {
                    const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total)
                    onProgress(progress)
                }
            },
        })
        return response.data
    },

    /**
     * Get restore job status
     */
    async getRestoreStatus(jobId: string): Promise<RestoreJob> {
        const response = await api.get(`/restore/jobs/${jobId}`)
        return response.data
    },

    /**
     * Validate a backup without restoring
     */
    async validateBackup(backupId: string): Promise<{
        valid: boolean
        errors: string[]
        warnings: string[]
    }> {
        const response = await api.post(`/backup/${backupId}/validate`)
        return response.data
    },

    /**
     * Get backup statistics
     */
    async getBackupStats(): Promise<{
        total_backups: number
        total_size_bytes: number
        last_backup_at?: string
        oldest_backup_at?: string
    }> {
        const response = await api.get('/backup/stats')
        return response.data
    },
}
