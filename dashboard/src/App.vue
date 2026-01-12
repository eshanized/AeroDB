<script setup lang="ts">
import { RouterView } from 'vue-router'
import { onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

onMounted(() => {
  // Initialize auth store (fetch user if token exists)
  authStore.initialize()
})
</script>

<template>
  <RouterView />
</template>

<style scoped>
</style>
