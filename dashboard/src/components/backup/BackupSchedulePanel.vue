<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { backupService } from '@/services/backup'

const schedule = ref<{
    enabled: boolean
    cron_expression: string
    retention_days: number
    compression: 'none' | 'gzip' | 'zstd'
    incremental: boolean
    last_run_at?: string
    next_run_at?: string
} | null>(null)

const history = ref<Array<{
    id: string
    type: 'manual' | 'scheduled'
    status: 'success' | 'failed'
    started_at: string
    completed_at?: string
    size_bytes?: number
    error?: string
}>>([])

const loading = ref(false)
const saving = ref(false)
const isEditing = ref(false)

// Edit form
const editForm = ref({
    enabled: true,
    cron_expression: '0 0 * * *',
    retention_days: 30,
    compression: 'zstd' as 'none' | 'gzip' | 'zstd',
    incremental: true,
})

const cronPresets = [
    { label: 'Every hour', value: '0 * * * *' },
    { label: 'Every day at midnight', value: '0 0 * * *' },
    { label: 'Every Sunday', value: '0 0 * * 0' },
    { label: 'Every month', value: '0 0 1 * *' },
]

onMounted(async () => {
    loading.value = true
    try {
        const [scheduleData, historyData] = await Promise.all([
            backupService.getSchedule(),
            backupService.getBackupHistory({ limit: 10 }),
        ])
        schedule.value = scheduleData
        history.value = historyData
        
        // Initialize edit form
        editForm.value = { ...scheduleData }
    } finally {
        loading.value = false
    }
})

const saveSchedule = async () => {
    saving.value = true
    try {
        await backupService.updateSchedule(editForm.value)
        schedule.value = await backupService.getSchedule()
        isEditing.value = false
    } finally {
        saving.value = false
    }
}

const runNow = async () => {
    try {
        await backupService.runScheduledBackup()
        history.value = await backupService.getBackupHistory({ limit: 10 })
    } catch (e) {
        console.error('Failed to run backup', e)
    }
}

const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
}

const formatDate = (ts: string) => new Date(ts).toLocaleString()
</script>

<template>
    <div class="space-y-6">
        <div class="flex items-center justify-between">
            <h2 class="font-semibold">Backup Schedule</h2>
            <div class="flex gap-2">
                <button
                    @click="runNow"
                    class="px-3 py-1 text-sm border border-border rounded hover:bg-muted"
                >
                    Run Now
                </button>
                <button
                    v-if="!isEditing"
                    @click="isEditing = true"
                    class="px-3 py-1 text-sm bg-primary text-primary-foreground rounded hover:opacity-90"
                >
                    Edit
                </button>
            </div>
        </div>
        
        <div v-if="loading" class="text-center py-8 text-muted-foreground">
            Loading...
        </div>
        
        <!-- View mode -->
        <div v-else-if="!isEditing && schedule" class="p-4 bg-card border border-border rounded-lg space-y-3">
            <div class="flex items-center justify-between">
                <span class="text-muted-foreground">Status</span>
                <span :class="schedule.enabled ? 'text-green-500' : 'text-muted-foreground'">
                    {{ schedule.enabled ? '● Enabled' : '○ Disabled' }}
                </span>
            </div>
            <div class="flex items-center justify-between">
                <span class="text-muted-foreground">Schedule</span>
                <span class="font-mono">{{ schedule.cron_expression }}</span>
            </div>
            <div class="flex items-center justify-between">
                <span class="text-muted-foreground">Retention</span>
                <span>{{ schedule.retention_days }} days</span>
            </div>
            <div class="flex items-center justify-between">
                <span class="text-muted-foreground">Compression</span>
                <span class="uppercase">{{ schedule.compression }}</span>
            </div>
            <div class="flex items-center justify-between">
                <span class="text-muted-foreground">Incremental</span>
                <span>{{ schedule.incremental ? 'Yes' : 'No' }}</span>
            </div>
            <div v-if="schedule.next_run_at" class="flex items-center justify-between pt-2 border-t border-border">
                <span class="text-muted-foreground">Next Run</span>
                <span>{{ formatDate(schedule.next_run_at) }}</span>
            </div>
        </div>
        
        <!-- Edit mode -->
        <div v-else-if="isEditing" class="p-4 bg-card border border-border rounded-lg space-y-4">
            <div>
                <label class="flex items-center gap-2 cursor-pointer">
                    <input type="checkbox" v-model="editForm.enabled" />
                    <span class="font-medium">Enable scheduled backups</span>
                </label>
            </div>
            
            <div>
                <label class="block text-sm font-medium mb-1">Cron Expression</label>
                <input
                    v-model="editForm.cron_expression"
                    type="text"
                    class="w-full px-3 py-2 bg-background border border-input rounded-md font-mono"
                />
                <div class="flex gap-2 mt-2">
                    <button
                        v-for="preset in cronPresets"
                        :key="preset.value"
                        @click="editForm.cron_expression = preset.value"
                        class="px-2 py-1 text-xs bg-muted rounded hover:bg-muted/80"
                    >
                        {{ preset.label }}
                    </button>
                </div>
            </div>
            
            <div>
                <label class="block text-sm font-medium mb-1">Retention (days)</label>
                <input
                    v-model.number="editForm.retention_days"
                    type="number"
                    min="1"
                    class="w-full px-3 py-2 bg-background border border-input rounded-md"
                />
            </div>
            
            <div>
                <label class="block text-sm font-medium mb-1">Compression</label>
                <select v-model="editForm.compression" class="w-full px-3 py-2 bg-background border border-input rounded-md">
                    <option value="none">None (fastest)</option>
                    <option value="gzip">GZIP (balanced)</option>
                    <option value="zstd">ZSTD (best compression)</option>
                </select>
            </div>
            
            <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="editForm.incremental" />
                <span>Use incremental backups</span>
            </label>
            
            <div class="flex justify-end gap-3 pt-2">
                <button @click="isEditing = false" class="px-4 py-2 border border-border rounded">
                    Cancel
                </button>
                <button
                    @click="saveSchedule"
                    :disabled="saving"
                    class="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
                >
                    {{ saving ? 'Saving...' : 'Save' }}
                </button>
            </div>
        </div>
        
        <!-- Backup history -->
        <div v-if="history.length > 0">
            <h3 class="font-medium mb-3">Recent Backups</h3>
            <div class="space-y-2">
                <div
                    v-for="item in history"
                    :key="item.id"
                    class="flex items-center justify-between p-3 bg-muted/50 rounded-lg text-sm"
                >
                    <div class="flex items-center gap-3">
                        <span :class="item.status === 'success' ? 'text-green-500' : 'text-red-500'">
                            {{ item.status === 'success' ? '✓' : '✗' }}
                        </span>
                        <span class="px-2 py-0.5 bg-muted rounded text-xs">
                            {{ item.type }}
                        </span>
                        <span class="text-muted-foreground">{{ formatDate(item.started_at) }}</span>
                    </div>
                    <span v-if="item.size_bytes" class="text-muted-foreground">
                        {{ formatBytes(item.size_bytes) }}
                    </span>
                </div>
            </div>
        </div>
    </div>
</template>
