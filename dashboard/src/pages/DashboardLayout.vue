<template>
  <div class="min-h-screen flex">
    <!-- Sidebar -->
    <aside class="w-64 bg-gray-900 text-white flex flex-col">
      <div class="p-4">
        <h1 class="text-2xl font-bold">AeroDB</h1>
      </div>
      
      <nav class="flex-1 px-2 py-4 space-y-1">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="flex items-center px-4 py-2 rounded-md hover:bg-gray-800 transition-colors"
          active-class="bg-gray-800"
        >
          {{ item.label }}
        </RouterLink>
      </nav>

      <div class="p-4 border-t border-gray-700">
        <button
          @click="authStore.logout()"
          class="w-full px-4 py-2 text-left hover:bg-gray-800 rounded-md transition-colors"
        >
          Logout
        </button>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 overflow-auto bg-gray-50 dark:bg-gray-900">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

const navItems = [
  { path: '/', label: 'Overview' },
  { path: '/database', label: 'Database' },
  { path: '/storage', label: 'Storage' },
  { path: '/observability', label: 'Observability' },
  { path: '/settings', label: 'Settings' }
]
</script>
