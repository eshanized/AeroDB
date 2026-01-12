import { useApi } from '@/composables/useApi'
import type { Bucket, StorageFile } from '@/types'

const { api } = useApi()

export const storageService = {
    /**
     * Get all buckets
     */
    async getBuckets(): Promise<Bucket[]> {
        const response = await api.get('/storage/buckets')
        return response.data
    },

    /**
     * Get a specific bucket
     */
    async getBucket(bucketName: string): Promise<Bucket> {
        const response = await api.get(`/storage/buckets/${bucketName}`)
        return response.data
    },

    /**
     * Create a new bucket
     */
    async createBucket(name: string, isPublic: boolean = false): Promise<Bucket> {
        const response = await api.post('/storage/buckets', {
            name,
            public: isPublic,
        })
        return response.data
    },

    /**
     * Delete a bucket
     */
    async deleteBucket(bucketName: string): Promise<void> {
        await api.delete(`/storage/buckets/${bucketName}`)
    },

    /**
     * Update bucket settings
     */
    async updateBucket(bucketName: string, isPublic: boolean): Promise<Bucket> {
        const response = await api.patch(`/storage/buckets/${bucketName}`, {
            public: isPublic,
        })
        return response.data
    },

    /**
     * List files in a bucket
     */
    async listFiles(bucketName: string, path: string = ''): Promise<StorageFile[]> {
        const params = new URLSearchParams()
        if (path) params.append('path', path)

        const response = await api.get(`/storage/buckets/${bucketName}/files?${params}`)
        return response.data
    },

    /**
     * Get file metadata
     */
    async getFile(bucketName: string, filePath: string): Promise<StorageFile> {
        const response = await api.get(`/storage/buckets/${bucketName}/files/${encodeURIComponent(filePath)}`)
        return response.data
    },

    /**
     * Upload a file
     */
    async uploadFile(
        bucketName: string,
        path: string,
        file: File,
        onProgress?: (progress: number) => void
    ): Promise<StorageFile> {
        const formData = new FormData()
        formData.append('file', file)
        formData.append('path', path)

        const response = await api.post(`/storage/buckets/${bucketName}/files`, formData, {
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
     * Delete a file
     */
    async deleteFile(bucketName: string, filePath: string): Promise<void> {
        await api.delete(`/storage/buckets/${bucketName}/files/${encodeURIComponent(filePath)}`)
    },

    /**
     * Move/rename a file
     */
    async moveFile(bucketName: string, fromPath: string, toPath: string): Promise<StorageFile> {
        const response = await api.post(`/storage/buckets/${bucketName}/files/move`, {
            from: fromPath,
            to: toPath,
        })
        return response.data
    },

    /**
     * Create a signed URL for temporary access
     */
    async createSignedUrl(
        bucketName: string,
        filePath: string,
        expiresIn: number = 3600
    ): Promise<{ url: string; expires_at: string }> {
        const response = await api.post(`/storage/buckets/${bucketName}/files/${encodeURIComponent(filePath)}/sign`, {
            expires_in: expiresIn,
        })
        return response.data
    },

    /**
     * Get public URL for a file (only works for public buckets)
     */
    getPublicUrl(bucketName: string, filePath: string): string {
        const { baseURL } = useApi()
        return `${baseURL}/storage/buckets/${bucketName}/public/${filePath}`
    },

    /**
     * Get bucket statistics
     */
    async getBucketStats(bucketName: string): Promise<{
        total_files: number
        total_size: number
    }> {
        const response = await api.get(`/storage/buckets/${bucketName}/stats`)
        return response.data
    },
}
