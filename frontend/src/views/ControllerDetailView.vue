<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import { useControllersStore } from '@/stores/controllers'
import ReadingsTable from '@/components/ReadingsTable.vue'

const AUTO_REFRESH_MS = 2 * 60 * 1000

const props = defineProps({
  id: { type: String, required: true },
})

const store = useControllersStore()
const hours = ref(24)
let refreshInterval

onMounted(() => {
  store.fetchRecentReadings(Number(props.id), hours.value)
  refreshInterval = setInterval(() => {
    store.fetchRecentReadings(Number(props.id), hours.value, true)
  }, AUTO_REFRESH_MS)
})

onUnmounted(() => {
  clearInterval(refreshInterval)
})

function loadReadings() {
  store.fetchRecentReadings(Number(props.id), hours.value)
}

function formatDate(dateStr) {
  return new Date(dateStr).toLocaleString()
}

async function handleReclaimUuid() {
  if (
    !confirm(
      'Make this UUID available to a device again? The next device that claims a UUID will receive it.',
    )
  )
    return
  try {
    await store.reclaimUuid(Number(props.id))
    alert('UUID is ready to be claimed by a device.')
  } catch (e) {
    alert('Failed to reclaim UUID: ' + e.message)
  }
}

async function handleDelete() {
  if (!confirm('Are you sure you want to delete this controller?')) return
  try {
    await store.removeController(Number(props.id))
    window.location.href = '/'
  } catch (e) {
    alert('Failed to delete: ' + e.message)
  }
}
</script>

<template>
  <div class="detail-view" v-if="store.currentController">
    <div class="detail-header">
      <div>
        <RouterLink to="/" class="back-link">&larr; Back</RouterLink>
        <h1>{{ store.currentController.name }}</h1>
        <p class="location">{{ store.currentController.location }}</p>
      </div>
      <div class="header-actions">
        <button @click="handleReclaimUuid" class="btn btn-secondary">Reclaim UUID</button>
        <RouterLink :to="`/controller/${id}/edit`" class="btn btn-secondary">Edit</RouterLink>
        <button @click="handleDelete" class="btn btn-danger">Delete</button>
      </div>
    </div>

    <div class="info-cards">
      <div class="info-card">
        <span class="info-label">UUID</span>
        <span class="info-value mono">{{ store.currentController.uuid }}</span>
      </div>
      <div class="info-card">
        <span class="info-label">Created</span>
        <span class="info-value">{{ formatDate(store.currentController.created_at) }}</span>
      </div>
      <div class="info-card">
        <span class="info-label">Min Humidity</span>
        <span class="info-value"> {{ store.currentController.min_humidity ?? '—' }}% </span>
      </div>
      <div class="info-card">
        <span class="info-label">Max Humidity</span>
        <span class="info-value"> {{ store.currentController.max_humidity ?? '—' }}% </span>
      </div>
    </div>

    <div class="readings-section">
      <div class="readings-header">
        <h2>Humidity Readings</h2>
        <div class="hours-filter">
          <label for="hours">Last</label>
          <select id="hours" v-model="hours" @change="loadReadings">
            <option :value="1">1 hour</option>
            <option :value="6">6 hours</option>
            <option :value="12">12 hours</option>
            <option :value="24">24 hours</option>
            <option :value="48">48 hours</option>
            <option :value="168">7 days</option>
            <option :value="720">30 days</option>
          </select>
        </div>
      </div>

      <div v-if="store.loading" class="loading">Loading readings...</div>
      <div v-else-if="store.error" class="error">{{ store.error }}</div>
      <ReadingsTable
        v-else-if="store.currentController.readings?.length"
        :readings="store.currentController.readings"
        :min-humidity="store.currentController.min_humidity"
        :max-humidity="store.currentController.max_humidity"
      />
      <div v-else class="empty-state">No readings found for this period.</div>
    </div>
  </div>

  <div v-else-if="store.loading" class="loading">Loading controller...</div>
  <div v-else-if="store.error" class="error">
    {{ store.error }}
    <br />
    <RouterLink to="/" class="btn btn-secondary" style="margin-top: 1rem"
      >Back to Dashboard</RouterLink
    >
  </div>
</template>

<style scoped>
.back-link {
  display: inline-block;
  color: var(--color-text);
  opacity: 0.6;
  text-decoration: none;
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
}

.back-link:hover {
  opacity: 1;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1.5rem;
}

.detail-header h1 {
  font-size: 1.6rem;
  color: var(--color-heading);
}

.location {
  color: var(--color-text);
  opacity: 0.6;
  font-size: 0.9rem;
  margin-top: 0.25rem;
}

.header-actions {
  display: flex;
  gap: 0.5rem;
}

.info-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 0.75rem;
  margin-bottom: 2rem;
}

.info-card {
  background: var(--color-background-soft);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.info-label {
  display: block;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  opacity: 0.5;
  margin-bottom: 0.25rem;
}

.info-value {
  font-size: 0.95rem;
  color: var(--color-heading);
  word-break: break-all;
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Fira Mono', monospace;
  font-size: 0.8rem;
}

.readings-section h2 {
  font-size: 1.2rem;
  color: var(--color-heading);
  margin-bottom: 0.75rem;
}

.readings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.hours-filter {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
}

.hours-filter select {
  padding: 0.3rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-background);
  color: var(--color-text);
  font-size: 0.85rem;
}

.loading {
  text-align: center;
  padding: 2rem;
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
  padding: 2rem;
  opacity: 0.6;
}

.btn {
  display: inline-block;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  text-decoration: none;
  border: none;
  cursor: pointer;
  transition: background-color 0.2s;
}

.btn-secondary {
  background: var(--color-background-mute);
  color: var(--color-text);
}

.btn-secondary:hover {
  background: var(--color-border);
}

.btn-danger {
  background: #e74c3c;
  color: white;
}

.btn-danger:hover {
  background: #c0392b;
}
</style>
