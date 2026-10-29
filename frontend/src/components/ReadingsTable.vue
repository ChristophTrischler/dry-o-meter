<script setup>
const props = defineProps({
  readings: { type: Array, required: true },
  minHumidity: { type: [String, Number, null], default: null },
  maxHumidity: { type: [String, Number, null], default: null },
})

function formatTime(dateStr) {
  return new Date(dateStr).toLocaleString()
}

function readingClass(reading) {
  const h = Number(reading.humidity)
  const min = props.minHumidity != null ? Number(props.minHumidity) : null
  const max = props.maxHumidity != null ? Number(props.maxHumidity) : null

  if (min != null && h < min) return 'reading-low'
  if (max != null && h > max) return 'reading-high'
  return ''
}
</script>

<template>
  <div class="readings-table-wrapper">
    <table class="readings-table">
      <thead>
        <tr>
          <th>Time</th>
          <th>Humidity</th>
          <th>Battery</th>
          <th>RSSI</th>
          <th>Alert</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="reading in readings" :key="reading.time" :class="readingClass(reading)">
          <td>{{ formatTime(reading.time) }}</td>
          <td class="humidity-cell">{{ reading.humidity }}%</td>
          <td>{{ reading.battery_voltage != null ? `${reading.battery_voltage}V` : '—' }}</td>
          <td>{{ reading.rssi != null ? `${reading.rssi}dB` : '—' }}</td>
          <td>
            <span v-if="reading.is_alert" class="alert-badge">Alert</span>
            <span v-else class="ok-badge">OK</span>
          </td>
        </tr>
      </tbody>
    </table>
    <p v-if="!readings.length" class="no-readings">No readings to display.</p>
  </div>
</template>

<style scoped>
.readings-table-wrapper {
  overflow-x: auto;
}

.readings-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85rem;
}

.readings-table th {
  text-align: left;
  padding: 0.6rem 0.75rem;
  border-bottom: 2px solid var(--color-border);
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  opacity: 0.5;
  color: var(--color-heading);
}

.readings-table td {
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--color-border);
}

.readings-table tbody tr:hover {
  background: var(--color-background-mute);
}

.reading-low td {
  border-left: 3px solid #f39c12;
}

.reading-high td {
  border-left: 3px solid #e74c3c;
}

.humidity-cell {
  font-weight: 500;
}

.reading-low .humidity-cell {
  color: #f39c12;
}

.reading-high .humidity-cell {
  color: #e74c3c;
}

.alert-badge {
  display: inline-block;
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  background: #fdecea;
  color: #e74c3c;
  font-size: 0.75rem;
  font-weight: 500;
}

.ok-badge {
  display: inline-block;
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  background: #e8f5e9;
  color: #27ae60;
  font-size: 0.75rem;
  font-weight: 500;
}

.no-readings {
  text-align: center;
  padding: 1.5rem;
  opacity: 0.5;
}
</style>
