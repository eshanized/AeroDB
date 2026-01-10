<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
    <div class="w-full max-w-md p-8 bg-white dark:bg-gray-800 rounded-lg shadow-xl">
      <div class="text-center mb-8">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">AeroDB</h1>
        <p class="text-gray-600 dark:text-gray-300 mt-2">Welcome back</p>
      </div>

      <div class="flex gap-2 mb-6">
        <button
          @click="mode = 'login'"
          :class="['flex-1 py-2 px-4 rounded-md transition-colors', mode === 'login' ? 'bg-blue-600 text-white' : 'bg-gray-100 text-gray-700']"
        >
          Login
        </button>
        <button
          @click="mode = 'signup'"
          :class="['flex-1 py-2 px-4 rounded-md transition-colors', mode === 'signup' ? 'bg-blue-600 text-white' : 'bg-gray-100 text-gray-700']"
        >
          Sign Up
        </button>
      </div>

      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Email</label>
          <input
            v-model="email"
            type="email"
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Password</label>
          <input
            v-model="password"
            type="password"
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <button
          type="submit"
          :disabled="loading"
          class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 disabled:opacity-50 transition-colors"
        >
          {{ loading ? 'Loading...' : mode === 'login' ? 'Login' : 'Sign Up' }}
        </button>

        <p v-if="error" class="text-red-600 text-sm text-center">{{ error }}</p>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useRouter } from 'vue-router'

const router = useRouter()
const authStore = useAuthStore()

const mode = ref<'login' | 'signup'>('login')
const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')

async function handleSubmit() {
  loading.value = true
  error.value = ''

  const result = mode.value === 'login'
    ? await authStore.login(email.value, password.value)
    : await authStore.signup(email.value, password.value)

  loading.value = false

  if (result.success) {
    router.push('/')
  } else {
    error.value = result.error || 'An error occurred'
  }
}
</script>
