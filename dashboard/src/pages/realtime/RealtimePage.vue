<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import AppLayout from '@/components/layout/AppLayout.vue'
import { useApi } from '@/composables/useApi'
import { useWebSocket } from '@/composables/useWebSocket'
import type { Subscription } from '@/types'

const { api } = useApi()
const { connected, events, error: wsError, connect, disconnect } = useWebSocket()

const liveUpdatesEnabled = ref(false)

// Fetch active subscriptions
const { data: subscriptions, isLoading, error } = useQuery({
  queryKey: ['subscriptions'],
  queryFn: async () => {
    const { data } = await api!.get('/realtime/v1/subscriptions')
    return data as Subscription[]
  },
})

const toggleLiveUpdates = () => {
  if (liveUpdatesEnabled.value) {
    disconnect()
    liveUpdatesEnabled.value = false
  } else {
    connect()
    liveUpdatesEnabled.value = true
  }
}

onMounted(() => {
  // Auto-connect on mount
  connect()
  liveUpdatesEnabled.value = true
})

onUnmounted(() => {
  disconnect()
})

const formatTimestamp = (timestamp: string) => {
  return new Date(timestamp).toLocaleTimeString()
}
</script>

<template>
  <AppLayout>
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold">Real-Time</h1>
        <div class="flex items-center gap-4">
          <div :class="['px-3 py-1 rounded-full text-sm', connected ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400']">
            {{ connected ? '● Connected' : '○ Disconnected' }}
          </div>
          <button
            @click="toggleLiveUpdates"
            :class="[
              'px-4 py-2 rounded',
              liveUpdatesEnabled
                ? 'bg-destructive text-destructive-foreground'
                : 'bg-primary text-primary-foreground'
            ]"
          >
            {{ liveUpdatesEnabled ? 'Disable Live Updates' : 'Enable Live Updates' }}
          </button>
        </div>
      </div>

      <!-- Active Subscriptions -->
      <div class="space-y-4">
        <h2 class="text-xl font-semibold">Active Subscriptions</h2>
        
        <div v-if="isLoading" class="text-center py-8">
          <p class="text-muted-foreground">Loading subscriptions...</p>
        </div>

        <div v-else-if="error" class="p-4 bg-destructive/10 text-destructive rounded-lg">
          Failed to load subscriptions: {{ error }}
        </div>

        <div v-else-if="!subscriptions || subscriptions.length === 0" class="text-center py-8">
          <p class="text-muted-foreground">No active subscriptions</p>
        </div>

        <div v-else class="border border-border rounded-lg overflow-hidden">
          <table class="w-full">
            <thead class="bg-muted">
              <tr>
                <th class="px-4 py-3 text-left text-sm font-medium">User</th>
                <th class="px-4 py-3 text-left text-sm font-medium">Channel</th>
                <th class="px-4 py-3 text-left text-sm font-medium">Filter</th>
                <th class="px-4 py-3 text-left text-sm font-medium">Connected At</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="sub in subscriptions"
                :key="sub.id"
                class="border-t border-border hover:bg-secondary/50"
              >
                <td class="px-4 py-3 text-sm">{{ sub.user_id }}</td>
                <td class="px-4 py-3 text-sm font-mono">{{ sub.channel }}</td>
                <td class="px-4 py-3 text-sm font-mono text-muted-foreground">
                  {{ sub.filter ? JSON.stringify(sub.filter) : '-' }}
                </td>
                <td class="px-4 py-3 text-sm text-muted-foreground">
                  {{ new Date(sub.connected_at).toLocaleString() }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Live Events -->
      <div class="space-y-4">
        <h2 class="text-xl font-semibold">Live Events</h2>
        
        <div v-if="wsError" class="p-4 bg-destructive/10 text-destructive rounded-lg">
          WebSocket Error: {{ wsError }}
        </div>

        <div v-if="!connected" class="text-center py-8">
          <p class="text-muted-foreground">Enable live updates to see events</p>
        </div>

        <div v-else-if="events.length === 0" class="text-center py-8">
          <p class="text-muted-foreground">Waiting for events...</p>
        </div>

        <div v-else class="space-y-2 max-h-96 overflow-y-auto">
          <div
            v-for="(event, idx) in events"
            :key="idx"
            class="p-4 bg-card border border-border rounded-lg"
          >
            <div class="flex items-center justify-between mb-2">
              <span class="font-semibold">{{ event.type }}</span>
              <span class="text-sm text-muted-foreground">{{ formatTimestamp(event.timestamp) }}</span>
            </div>
            <pre class="text-sm text-muted-foreground overflow-x-auto">{{ JSON.stringify(event.payload, null, 2) }}</pre>
          </div>
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<style scoped>
</style>
