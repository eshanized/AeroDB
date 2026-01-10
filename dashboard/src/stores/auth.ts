import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import axios from 'axios'

export const useAuthStore = defineStore('auth', () => {
    const token = ref<string | null>(localStorage.getItem('token'))
    const user = ref<{ id: string; email: string } | null>(null)

    const isAuthenticated = computed(() => !!token.value)

    async function login(email: string, password: string) {
        try {
            const response = await axios.post('/auth/login', { email, password })
            token.value = response.data.token
            user.value = response.data.user
            localStorage.setItem('token', response.data.token)
            return { success: true }
        } catch (error) {
            return { success: false, error: 'Login failed' }
        }
    }

    async function signup(email: string, password: string) {
        try {
            const response = await axios.post('/auth/signup', { email, password })
            token.value = response.data.token
            user.value = response.data.user
            localStorage.setItem('token', response.data.token)
            return { success: true }
        } catch (error) {
            return { success: false, error: 'Signup failed' }
        }
    }

    function logout() {
        token.value = null
        user.value = null
        localStorage.removeItem('token')
    }

    return {
        token,
        user,
        isAuthenticated,
        login,
        signup,
        logout
    }
})
