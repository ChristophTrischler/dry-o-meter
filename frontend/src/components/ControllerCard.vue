<script setup>
import { computed } from 'vue'

const props = defineProps({
  controller: { type: Object, required: true },
})

const latestReading = computed(() => {
  return props.controller.readings?.[0] ?? null
})

const humidityClass = computed(() => {
  if (!latestReading.value) return ''
  const h = Number(latestReading.value.humidity)
  const min = props.controller.min_humidity != null ? Number(props.controller.min_humidity) : null
  const max = props.controller.max_humidity != null ? Number(props.controller.max_humidity) : null

  if (min != null && h < min) return 'humidity-low'
  if (max != null && h > max) return 'humidity-high'
  return 'humidity-ok'
})

function formatTime(dateStr) {
  if (!dateStr) return '—'
  return new Date(dateStr).toLocaleString()
}
</script>

<template>
  <RouterLink :to="`/controller/${controller.controller_id}`" class="card-link">
    <div class="controller-card">
      <div class="card-header">
        <h3 class="card-title">{{ controller.name }}</h3>
        <span v-if="latestReading" :class="['status-dot', humidityClass]"></span>
      </div>

      <p class="card-location">{{ controller.location }}</p>

      <div class="card-stats">
        <div v-if="latestReading" class="stat">
          <span class="stat-label">Humidity</span>
          <span :class="['stat-value', humidityClass]">{{ latestReading.humidity }}%</span>
        </div>
        <div v-if="latestReading?.battery_voltage" class="stat">
          <span class="stat-label">Battery</span>
          <span class="stat-value">{{ latestReading.battery_voltage }}V</span>
        </div>
        <div v-if="latestReading?.rssi" class="stat">
          <span class="stat-label">RSSI</span>
          <span class="stat-value">{{ latestReading.rssi }}dB</span>
        </div>
      </div>

      <div class="card-footer" v-if="latestReading">
        <span class="last-seen">Last reading: {{ formatTime(latestReading.time) }}</span>
      </div>
      <div class="card-footer" v-else>
        <span class="last-seen no-readings">No readings yet</span>
      </div>
    </div>
  </RouterLink>
</template>

<style scoped>
.card-link {
  text-decoration: none;
  color: inherit;
  display: block;
}

.controller-card {
  background: var(--color-background-soft);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 1rem 1.25rem;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.controller-card:hover {
  border-color: var(--color-border-hover);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.25rem;
}

.card-title {
  font-size: 1.05rem;
  color: var(--color-heading);
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.humidity-ok {
  color: #27ae60;
}

.humidity-low {
  color: #f39c12;
}

.humidity-high {
  color: #e74c3c;
}

.status-dot.humidity-ok {
  background: #27ae60;
}

.status-dot.humidity-low {
  background: #f39c12;
}

.status-dot.humidity-high {
  background: #e74c3c;
}

.card-location {
  font-size: 0.85rem;
  opacity: 0.6;
  margin-bottom: 0.75rem;
}

.card-stats {
  display: flex;
  gap: 1.25rem;
  margin-bottom: 0.75rem;
}

.stat {
  display: flex;
  flex-direction: column;
}

.stat-label {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  opacity: 0.5;
}

.stat-value {
  font-size: 1.05rem;
  font-weight: 500;
  color: var(--color-heading);
}

.stat-value.humidity-low {
  color: #f39c12;
}

.stat-value.humidity-high {
  color: #e74c3c;
}

.card-footer {
  border-top: 1px solid var(--color-border);
  padding-top: 0.5rem;
}

.last-seen {
  font-size: 0.75rem;
  opacity: 0.5;
}

.no-readings {
  font-style: italic;
}
</style>
