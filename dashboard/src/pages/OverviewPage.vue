<template>
  <div class="p-8">
    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-6">Overview</h1>
    
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Database Status</h3>
        <p class="text-3xl font-bold text-green-600">Running</p>
      </div>

      <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Total Records</h3>
        <p class="text-3xl font-bold text-blue-600">0</p>
      </div>

      <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Storage Used</h3>
        <p class="text-3xl font-bold text-purple-600">0 MB</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Overview page will be populated with actual data later
</script>
