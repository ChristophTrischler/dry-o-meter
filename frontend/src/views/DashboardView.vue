<script setup>
import { onMounted } from 'vue'
import { useControllersStore } from '@/stores/controllers'
import ControllerCard from '@/components/ControllerCard.vue'

const store = useControllersStore()

onMounted(() => {
  store.fetchControllers()
})
</script>

<template>
  <div class="dashboard">
    <div class="dashboard-header">
      <h1>Controllers</h1>
      <p class="subtitle" v-if="store.controllerCount">
        {{ store.controllerCount }} device{{ store.controllerCount !== 1 ? 's' : '' }} registered
      </p>
    </div>

    <div v-if="store.loading && !store.controllers.length" class="loading">
      Loading controllers...
    </div>

    <div v-else-if="store.error" class="error">
      <p>Error: {{ store.error }}</p>
      <button @click="store.fetchControllers()" class="btn btn-secondary">Retry</button>
    </div>

    <div v-else-if="!store.controllers.length" class="empty-state">
      <div class="empty-icon">📡</div>
      <h2>No controllers yet</h2>
      <p>Add your first humidity controller to start monitoring.</p>
      <RouterLink to="/controller/new" class="btn btn-primary">Add Controller</RouterLink>
    </div>

    <div v-else class="controller-grid">
      <ControllerCard
        v-for="controller in store.controllers"
        :key="controller.controller_id"
        :controller="controller"
      />
    </div>
  </div>
</template>

<style scoped>
.dashboard-header {
  margin-bottom: 1.5rem;
}

.dashboard-header h1 {
  font-size: 1.6rem;
  color: var(--color-heading);
}

.subtitle {
  color: var(--color-text);
  opacity: 0.6;
  font-size: 0.9rem;
  margin-top: 0.25rem;
}

.loading {
  text-align: center;
  padding: 3rem;
  color: var(--color-text);
  opacity: 0.6;
}

.error {
  text-align: center;
  padding: 2rem;
  color: #e74c3c;
  background: #fdecea;
  border-radius: 8px;
}

.empty-state {
  text-align: center;
  padding: 4rem 2rem;
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.empty-state h2 {
  color: var(--color-heading);
  margin-bottom: 0.5rem;
}

.empty-state p {
  color: var(--color-text);
  opacity: 0.6;
  margin-bottom: 1.5rem;
}

.controller-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1rem;
}

.btn {
  display: inline-block;
  padding: 0.6rem 1.2rem;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 500;
  text-decoration: none;
  border: none;
  cursor: pointer;
  transition: background-color 0.2s;
}

.btn-primary {
  background: hsla(160, 100%, 37%, 1);
  color: white;
}

.btn-primary:hover {
  background: hsla(160, 100%, 32%, 1);
}

.btn-secondary {
  background: var(--color-background-mute);
  color: var(--color-text);
}

.btn-secondary:hover {
  background: var(--color-border);
}
</style>
