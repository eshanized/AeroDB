<template>
    <div class="restore-page">
        <div class="page-header">
            <h1>Restore from Backup</h1>
        </div>

        <div v-if="loading" class="loading">Loading...</div>

        <div v-else class="restore-content">
            <div class="restore-step">
                <h2>Step 1: Select Backup</h2>
                <div v-if="backups.length === 0" class="empty-state">
                    <p>No backups available</p>
                </div>
                <div v-else class="backup-list">
                    <div
                        v-for="backup in backups"
                        :key="backup.id"
                        class="backup-item"
                        :class="{ selected: selectedBackup?.id === backup.id }"
                        @click="selectedBackup = backup"
                    >
                        <div class="backup-info">
                            <strong>{{ backup.id.substring(0, 8) }}</strong>
                            <span>{{ new Date(backup.created_at).toLocaleString() }}</span>
                            <span>{{ formatBytes(backup.size_bytes) }}</span>
                        </div>
                    </div>
                </div>
                
                <div class="upload-section">
                    <p>Or upload a backup file:</p>
                    <input
                        type="file"
                        accept=".tar.gz,.tgz"
                        @change="handleFileUpload"
                    />
                    <div v-if="uploadProgress > 0" class="upload-progress">
                        <div class="progress-bar" :style="{ width: `${uploadProgress}%` }"></div>
                        <span>{{ uploadProgress }}%</span>
                    </div>
                </div>
            </div>

            <div v-if="selectedBackup" class="restore-step">
                <h2>Step 2: Validate Backup</h2>
                <button
                    class="btn-secondary"
                    @click="validateBackup"
                    :disabled="validating"
                >
                    {{ validating ? 'Validating...' : 'Validate Backup' }}
                </button>
                
                <div v-if="validationResult" class="validation-result">
                    <div v-if="validationResult.valid" class="success-message">
                        ✓ Backup is valid
                    </div>
                    <div v-else class="error-message">
                        ✗ Backup validation failed
                        <ul>
                            <li v-for="err in validationResult.errors" :key="err">{{ err }}</li>
                        </ul>
                    </div>
                    <div v-if="validationResult.warnings.length > 0" class="warning-message">
                        Warnings:
                        <ul>
                            <li v-for="warn in validationResult.warnings" :key="warn">{{ warn }}</li>
                        </ul>
                    </div>
                </div>
            </div>

            <div v-if="validationResult?.valid" class="restore-step">
                <h2>Step 3: Restore</h2>
                <div class="warning-box">
                    <strong>⚠️ Warning:</strong> Restoring will overwrite all current data. This action cannot be undone.
                </div>
                <button
                    class="btn-danger"
                    @click="performRestore"
                    :disabled="restoring"
                >
                    {{ restoring ? 'Restoring...' : 'Restore Database' }}
                </button>
            </div>

            <div v-if="restoreJob" class="restore-progress">
                <JobProgress
                    :job-name="`Restore ${selectedBackup?.id.substring(0, 8)}`"
                    :status="restoreJob.status"
                    :progress="restoreJob.progress"
                    :error="restoreJob.error"
                    :message="getProgressMessage(restoreJob.status)"
                />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { backupService } from '@/services'
import { getErrorMessage } from '@/composables/useApi'
import type { RestoreJob } from '@/types'
import JobProgress from '@/components/common/JobProgress.vue'

const router = useRouter()
const route = useRoute()

const backups = ref<Array<{
    id: string
    manifest: any
    created_at: string
    size_bytes: number
}>>([])

const selectedBackup = ref<any>(null)
const validationResult = ref<any>(null)
const restoreJob = ref<RestoreJob | null>(null)

const loading = ref(false)
const validating = ref(false)
const restoring = ref(false)
const uploadProgress = ref(0)
const error = ref('')

const loadBackups = async () => {
    loading.value = true
    try {
        backups.value = await backupService.listBackups()
        
        // Auto-select if backup_id in query params
        const backupId = route.query.backup_id as string
        if (backupId) {
            selectedBackup.value = backups.value.find((b) => b.id === backupId)
        }
    } catch (err) {
        error.value = getErrorMessage(err)
    } finally {
        loading.value = false
    }
}

const handleFileUpload = async (event: Event) => {
    const file = (event.target as HTMLInputElement).files?.[0]
    if (!file) return

    try {
        const result = await backupService.uploadBackup(file, (progress) => {
            uploadProgress.value = progress
        })
        
        // Reload backups and select the uploaded one
        await loadBackups()
        selectedBackup.value = backups.value.find((b) => b.id === result.backup_id)
        uploadProgress.value = 0
    } catch (err) {
        error.value = getErrorMessage(err)
        uploadProgress.value = 0
    }
}

const validateBackup = async () => {
    if (!selectedBackup.value) return
    
    validating.value = true
    try {
        validationResult.value = await backupService.validateBackup(selectedBackup.value.id)
    } catch (err) {
        error.value = getErrorMessage(err)
    } finally {
        validating.value = false
    }
}

const performRestore = async () => {
    if (!selectedBackup.value) return
    if (!confirm('Are you absolutely sure? This will overwrite all current data!')) return
    
    restoring.value = true
    try {
        const job = await backupService.restoreBackup(selectedBackup.value.id)
        restoreJob.value = job
        
        // Poll for job status
        const pollInterval = setInterval(async () => {
            try {
                const updatedJob = await backupService.getRestoreStatus(job.id)
                restoreJob.value = updatedJob
                
                if (updatedJob.status === 'completed' || updatedJob.status === 'failed') {
                    clearInterval(pollInterval)
                    restoring.value = false
                    
                    if (updatedJob.status === 'completed') {
                        alert('Restore completed successfully!')
                        router.push('/')
                    }
                }
            } catch (err) {
                clearInterval(pollInterval)
                restoring.value = false
                error.value = getErrorMessage(err)
            }
        }, 1000)
    } catch (err) {
        restoring.value = false
        error.value = getErrorMessage(err)
    }
}

const getProgressMessage = (status: string): string => {
    const messages: Record<string, string> = {
        pending: 'Waiting to start...',
        validating: 'Validating backup integrity...',
        restoring: 'Restoring data...',
        completed: 'Restore completed!',
        failed: 'Restore failed',
    }
    return messages[status] || ''
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
.restore-page {
    padding: 2rem;
    max-width: 800px;
    margin: 0 auto;
}

.page-header h1 {
    font-size: 1.875rem;
    font-weight: 700;
    margin-bottom: 2rem;
}

.restore-content {
    display: flex;
    flex-direction: column;
    gap: 2rem;
}

.restore-step {
    padding: 1.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: var(--color-bg-secondary);
}

.restore-step h2 {
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: 1rem;
}

.backup-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1rem;
}

.backup-item {
    padding: 1rem;
    border: 2px solid var(--color-border);
    border-radius: 0.375rem;
    cursor: pointer;
    transition: all 0.2s;
}

.backup-item:hover {
    border-color: var(--color-primary);
}

.backup-item.selected {
    border-color: var(--color-primary);
    background: rgba(59, 130, 246, 0.1);
}

.backup-info {
    display: flex;
    gap: 1rem;
}

.upload-section {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--color-border);
}

.upload-progress {
    margin-top: 0.5rem;
    height: 8px;
    background: var(--color-bg);
    border-radius: 9999px;
    position: relative;
}

.progress-bar {
    height: 100%;
    background: var(--color-primary);
    border-radius: 9999px;
    transition: width 0.3s;
}

.validation-result {
    margin-top: 1rem;
}

.success-message {
    padding: 0.75rem;
    background: #d1fae5;
    color: #065f46;
    border-radius: 0.375rem;
}

.error-message,
.warning-message {
    padding: 0.75rem;
    border-radius: 0.375rem;
    margin-bottom: 0.5rem;
}

.error-message {
    background: #fee2e2;
    color: #991b1b;
}

.warning-message {
    background: #fef3c7;
    color: #92400e;
}

.warning-box {
    padding: 1rem;
    background: #fef3c7;
    color: #92400e;
    border-radius: 0.375rem;
    margin-bottom: 1rem;
}

.btn-secondary,
.btn-danger {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-weight: 500;
    cursor: pointer;
}

.btn-secondary {
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text);
}

.btn-danger {
    background: #ef4444;
    color: white;
    border: none;
}

.btn-secondary:disabled,
.btn-danger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.empty-state {
    padding: 2rem;
    text-align: center;
    color: var(--color-text-muted);
}

.loading {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
}
</style>
