<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { snapshotService } from '@/services/snapshot'

const emit = defineEmits<{
    (e: 'close'): void
    (e: 'complete'): void
}>()

const step = ref(1)
const loading = ref(false)
const error = ref<string | null>(null)

// Step 1: Select target time
const pitrRange = ref<{
    oldest_available: string
    latest_available: string
    wal_retention_hours: number
} | null>(null)

const targetDate = ref('')
const targetTime = ref('')

// Step 2: Validation
const validationStatus = ref<'pending' | 'validating' | 'valid' | 'invalid'>('pending')

// Step 3: Restore progress
const restoreJob = ref<{
    job_id: string
    status: 'pending' | 'running' | 'complete' | 'failed'
    progress_percent: number
} | null>(null)

const selectedDateTime = computed(() => {
    if (!targetDate.value || !targetTime.value) return null
    return new Date(`${targetDate.value}T${targetTime.value}:00`).toISOString()
})

onMounted(async () => {
    loading.value = true
    try {
        pitrRange.value = await snapshotService.getPITRRange()
        
        // Set default to latest available
        if (pitrRange.value?.latest_available) {
            const latest = new Date(pitrRange.value.latest_available)
            targetDate.value = latest.toISOString().split('T')[0]
            targetTime.value = latest.toTimeString().slice(0, 5)
        }
    } catch (e) {
        error.value = 'Failed to load PITR range'
    } finally {
        loading.value = false
    }
})

const validateRestore = async () => {
    if (!selectedDateTime.value) return
    
    step.value = 2
    validationStatus.value = 'validating'
    
    try {
        await snapshotService.restoreToPointInTime(selectedDateTime.value, { validate_only: true })
        validationStatus.value = 'valid'
    } catch (e) {
        validationStatus.value = 'invalid'
        error.value = String(e)
    }
}

const executeRestore = async () => {
    if (!selectedDateTime.value) return
    
    step.value = 3
    
    try {
        const result = await snapshotService.restoreToPointInTime(selectedDateTime.value)
        restoreJob.value = {
            job_id: result.job_id,
            status: result.status,
            progress_percent: 0,
        }
        
        // Poll for status
        await pollRestoreStatus(result.job_id)
    } catch (e) {
        error.value = String(e)
    }
}

const pollRestoreStatus = async (jobId: string) => {
    while (true) {
        const status = await snapshotService.getPITRJobStatus(jobId)
        restoreJob.value = {
            job_id: jobId,
            status: status.status,
            progress_percent: status.progress_percent,
        }
        
        if (status.status === 'complete' || status.status === 'failed') {
            if (status.status === 'failed') {
                error.value = status.error || 'Restore failed'
            }
            break
        }
        
        await new Promise(resolve => setTimeout(resolve, 1000))
    }
}

const formatDate = (ts: string) => new Date(ts).toLocaleString()
</script>

<template>
    <div
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        @click="emit('close')"
    >
        <div
            class="bg-card border border-border rounded-lg shadow-lg max-w-lg w-full mx-4 p-6"
            @click.stop
        >
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-lg font-semibold">Point-in-Time Recovery</h2>
                <button @click="emit('close')" class="p-2 hover:bg-muted rounded">✕</button>
            </div>
            
            <!-- Steps indicator -->
            <div class="flex items-center gap-2 mb-6">
                <div v-for="s in 3" :key="s" class="flex items-center gap-2">
                    <div
                        class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium"
                        :class="step >= s ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'"
                    >
                        {{ s }}
                    </div>
                    <div v-if="s < 3" class="w-8 h-0.5" :class="step > s ? 'bg-primary' : 'bg-muted'"></div>
                </div>
            </div>
            
            <!-- Step 1: Select time -->
            <div v-if="step === 1">
                <h3 class="font-medium mb-4">Select Recovery Point</h3>
                
                <div v-if="loading" class="text-center py-4 text-muted-foreground">
                    Loading...
                </div>
                
                <div v-else-if="pitrRange" class="space-y-4">
                    <div class="p-3 bg-muted/50 rounded-lg text-sm">
                        <p class="text-muted-foreground mb-1">Available recovery window:</p>
                        <p class="font-mono">
                            {{ formatDate(pitrRange.oldest_available) }} — {{ formatDate(pitrRange.latest_available) }}
                        </p>
                        <p class="text-xs text-muted-foreground mt-1">
                            ({{ pitrRange.wal_retention_hours }} hours of WAL retained)
                        </p>
                    </div>
                    
                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class="block text-sm font-medium mb-1">Date</label>
                            <input
                                v-model="targetDate"
                                type="date"
                                class="w-full px-3 py-2 bg-background border border-input rounded-md"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium mb-1">Time</label>
                            <input
                                v-model="targetTime"
                                type="time"
                                class="w-full px-3 py-2 bg-background border border-input rounded-md"
                            />
                        </div>
                    </div>
                    
                    <div class="p-3 bg-yellow-500/10 text-yellow-600 rounded-lg text-sm">
                        ⚠️ Warning: This will restore the database to the selected point in time. All changes after this point will be lost.
                    </div>
                </div>
                
                <button
                    @click="validateRestore"
                    :disabled="!selectedDateTime"
                    class="w-full mt-4 px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
                >
                    Continue
                </button>
            </div>
            
            <!-- Step 2: Validation -->
            <div v-if="step === 2">
                <h3 class="font-medium mb-4">Validation</h3>
                
                <div class="py-8 text-center">
                    <div v-if="validationStatus === 'validating'" class="space-y-4">
                        <div class="text-4xl animate-spin">⟳</div>
                        <p>Validating recovery point...</p>
                    </div>
                    <div v-else-if="validationStatus === 'valid'" class="space-y-4">
                        <div class="text-4xl text-green-500">✓</div>
                        <p class="text-green-500 font-medium">Recovery Point Valid</p>
                        <p class="text-sm text-muted-foreground">
                            The selected time point is available for recovery.
                        </p>
                    </div>
                    <div v-else-if="validationStatus === 'invalid'" class="space-y-4">
                        <div class="text-4xl text-red-500">✗</div>
                        <p class="text-red-500 font-medium">Validation Failed</p>
                        <p class="text-sm text-muted-foreground">{{ error }}</p>
                    </div>
                </div>
                
                <div class="flex gap-3">
                    <button @click="step = 1" class="flex-1 px-4 py-2 border border-border rounded">
                        Back
                    </button>
                    <button
                        @click="executeRestore"
                        :disabled="validationStatus !== 'valid'"
                        class="flex-1 px-4 py-2 bg-destructive text-destructive-foreground rounded hover:opacity-90 disabled:opacity-50"
                    >
                        Restore Now
                    </button>
                </div>
            </div>
            
            <!-- Step 3: Restore -->
            <div v-if="step === 3">
                <h3 class="font-medium mb-4">Restore Progress</h3>
                
                <div class="py-8 text-center">
                    <div v-if="restoreJob?.status === 'running' || restoreJob?.status === 'pending'" class="space-y-4">
                        <p class="mb-2">Restoring database...</p>
                        <div class="w-full bg-muted rounded-full h-3">
                            <div
                                class="bg-primary h-3 rounded-full transition-all duration-300"
                                :style="{ width: `${restoreJob.progress_percent}%` }"
                            ></div>
                        </div>
                        <p class="text-sm text-muted-foreground">{{ restoreJob.progress_percent }}% complete</p>
                    </div>
                    <div v-else-if="restoreJob?.status === 'complete'" class="space-y-4">
                        <div class="text-4xl text-green-500">✓</div>
                        <p class="text-green-500 font-medium">Recovery Complete</p>
                        <p class="text-sm text-muted-foreground">
                            Database restored to {{ formatDate(selectedDateTime!) }}
                        </p>
                    </div>
                    <div v-else-if="restoreJob?.status === 'failed'" class="space-y-4">
                        <div class="text-4xl text-red-500">✗</div>
                        <p class="text-red-500 font-medium">Recovery Failed</p>
                        <p class="text-sm text-muted-foreground">{{ error }}</p>
                    </div>
                </div>
                
                <button
                    @click="restoreJob?.status === 'complete' ? emit('complete') : emit('close')"
                    :disabled="restoreJob?.status === 'running' || restoreJob?.status === 'pending'"
                    class="w-full mt-4 px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
                >
                    {{ restoreJob?.status === 'complete' ? 'Done' : 'Close' }}
                </button>
            </div>
        </div>
    </div>
</template>
