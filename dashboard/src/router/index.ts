import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/login',
            name: 'Login',
            component: () => import('@/pages/Login.vue'),
            meta: { requiresAuth: false },
        },
        {
            path: '/',
            name: 'Home',
            component: () => import('@/pages/Home.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/database',
            name: 'Database',
            component: () => import('@/pages/database/DatabasePage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/database/table/:name',
            name: 'TableBrowser',
            component: () => import('@/pages/database/TableBrowser.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/database/sql',
            name: 'SQLConsole',
            component: () => import('@/pages/database/SQLConsole.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/auth',
            name: 'Auth',
            component: () => import('@/pages/auth/AuthPage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/storage',
            name: 'Storage',
            component: () => import('@/pages/storage/StoragePage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/storage/bucket/:name',
            name: 'FileBrowser',
            component: () => import('@/pages/storage/FileBrowser.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/realtime',
            name: 'Realtime',
            component: () => import('@/pages/realtime/RealtimePage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/functions',
            name: 'Functions',
            component: () => import('@/pages/functions/FunctionsPage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/functions/:id/edit',
            name: 'FunctionEditor',
            component: () => import('@/pages/functions/FunctionEditor.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/functions/:id/logs',
            name: 'FunctionLogs',
            component: () => import('@/pages/functions/FunctionLogs.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/backup',
            name: 'Backup',
            component: () => import('@/pages/backup/BackupPage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/restore',
            name: 'Restore',
            component: () => import('@/pages/backup/RestorePage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/cluster',
            name: 'Cluster',
            component: () => import('@/pages/cluster/ClusterPage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/logs',
            name: 'Logs',
            component: () => import('@/pages/observability/LogsPage.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/metrics',
            name: 'Metrics',
            component: () => import('@/pages/observability/MetricsPage.vue'),
            meta: { requiresAuth: true },
        },
    ],
})

// Navigation guard to check authentication
router.beforeEach((to, _from, next) => {
    const authStore = useAuthStore()

    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
        next('/login')
    } else if (to.path === '/login' && authStore.isAuthenticated) {
        next('/')
    } else {
        next()
    }
})

export default router
