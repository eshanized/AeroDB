import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/auth',
            name: 'auth',
            component: () => import('@/pages/AuthPage.vue')
        },
        {
            path: '/',
            component: () => import('@/pages/DashboardLayout.vue'),
            children: [
                {
                    path: '',
                    name: 'overview',
                    component: () => import('@/pages/OverviewPage.vue')
                },
                {
                    path: 'database',
                    name: 'database',
                    component: () => import('@/pages/DatabasePage.vue')
                },
                {
                    path: 'storage',
                    name: 'storage',
                    component: () => import('@/pages/StoragePage.vue')
                },
                {
                    path: 'observability',
                    name: 'observability',
                    component: () => import('@/pages/ObservabilityPage.vue')
                },
                {
                    path: 'settings',
                    name: 'settings',
                    component: () => import('@/pages/SettingsPage.vue')
                }
            ],
            meta: { requiresAuth: true }
        }
    ]
})

// Navigation guard
router.beforeEach((to, _from, next) => {
    const authStore = useAuthStore()

    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
        next({ name: 'auth' })
    } else if (to.name === 'auth' && authStore.isAuthenticated) {
        next({ name: 'overview' })
    } else {
        next()
    }
})

export default router
