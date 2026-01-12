<template>
    <div class="backup-page">
        <div class="page-header">
            <h1>Backups</h1>
            <button class="btn-primary" @click="createBackup" :disabled="creating">
                {{ creating ? 'Creating...' : '+ Create Backup' }}
            </button>
        </div>

        <div v-if="loading" class="loading">Loading backups...</div>

        <div v-else-if="error" class="error-state">
            <p>{{ error }}</p>
            <button class="btn-secondary" @click="loadBackups">Retry</button>
        </div>

        <div v-else class="backup-content">
            <DataTable
                :columns="columns"
                :data="tableData"
                :loading="loading"
                @row-action="handleRowAction"
            />
        </div>

        <!-- Create Backup Job Progress -->
        <div v-if="currentJob" class="job-status">
            <JobProgress
                :job-name="`Backup ${currentJob.id}`"
                :status="currentJob.status"
                :progress="currentJob.progress"
                :error="currentJob.error"
            />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { backupService } from '@/services'
import { getErrorMessage } from '@/composables/useApi'
import type { BackupJob, TableRow } from '@/types'
import DataTable from '@/components/common/DataTable.vue'
import JobProgress from '@/components/common/JobProgress.vue'

const backups = ref<Array<{
    id: string
    manifest: any
    created_at: string
    size_bytes: number
}>>([])

const currentJob = ref<BackupJob | null>(null)
const loading = ref(false)
const creating = ref(false)
const error = ref('')

const columns = [
    { key: 'ID', header: 'ID' },
    { key: 'Created At', header: 'Created At' },
    { key: 'Size', header: 'Size' },
    { key: 'Tables', header: 'Tables' },
]

const tableData = computed((): TableRow[] => {
    return backups.value.map((backup): TableRow => ({
        'ID': backup.id.substring(0, 8),
        'Created At': new Date(backup.created_at).toLocaleString(),
        'Size': formatBytes(backup.size_bytes),
        'Tables': backup.manifest.tables.length.toString(),
        '_backup_data': backup,
    }))
})

const loadBackups = async () => {
    loading.value = true
    error.value = ''
    try {
        backups.value = await backupService.listBackups()
    } catch (err) {
        error.value = getErrorMessage(err)
    } finally {
        loading.value = false
    }
}

const createBackup = async () => {
    creating.value = true
    error.value = ''
    try {
        const job = await backupService.createBackup({
            compression: 'zstd',
        })
        currentJob.value = job
        
        // Poll for job status
        const pollInterval = setInterval(async () => {
            try {
                const updatedJob = await backupService.getBackupStatus(job.id)
                currentJob.value = updatedJob
                
                if (updatedJob.status === 'completed' || updatedJob.status === 'failed') {
                    clearInterval(pollInterval)
                    creating.value = false
                    if (updatedJob.status === 'completed') {
                        await loadBackups()
                    }
                }
            } catch (err) {
                clearInterval(pollInterval)
                creating.value = false
                error.value = getErrorMessage(err)
            }
        }, 1000)
    } catch (err) {
        creating.value = false
        error.value = getErrorMessage(err)
    }
}

const handleRowAction = async (action: string, rowData: any) => {
    const backup = rowData._actions.find((a: any) => a.action === action)?.data
    if (!backup) return

    if (action === 'download') {
        try {
            const blob = await backupService.downloadBackup(backup.id)
            const url = URL.createObjectURL(blob)
            const a = document.createElement('a')
            a.href = url
            a.download = `backup-${backup.id}.tar.gz`
            a.click()
            URL.revokeObjectURL(url)
        } catch (err) {
            error.value = getErrorMessage(err)
        }
    } else if (action === 'restore') {
        if (confirm(`Restore from backup ${backup.id}?\nThis will overwrite current data!`)) {
            // Navigate to restore page with backup ID
            window.location.href = `/restore?backup_id=${backup.id}`
        }
    } else if (action === 'delete') {
        if (confirm(`Delete backup ${backup.id}?`)) {
            try {
                await backupService.deleteBackup(backup.id)
                await loadBackups()
            } catch (err) {
                error.value = getErrorMessage(err)
            }
        }
    }
}

const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
}

onMounted(() => {
    loadBackups()
})
</script>

<style scoped>
.backup-page {
    padding: 2rem;
}

.page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
}

.page-header h1 {
    font-size: 1.875rem;
    font-weight: 700;
}

.backup-content {
    margin-bottom: 2rem;
}

.job-status {
    margin-top: 2rem;
}

.loading,
.error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    gap: 1rem;
}

.btn-primary {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-weight: 500;
    cursor: pointer;
    background: var(--color-primary);
    color: white;
    border: none;
}

.btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-secondary {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
}
</style>
