<script setup>
import { computed } from 'vue'

const props = defineProps({
  readings: { type: Array, required: true },
  minHumidity: { type: [String, Number, null], default: null },
  maxHumidity: { type: [String, Number, null], default: null },
})

const W = 640
const H = 260
const PAD = { top: 16, right: 16, bottom: 32, left: 36 }
const Y_MIN = 0
const Y_MAX = 100
const Y_TICKS = [0, 25, 50, 75, 100]
const X_TICK_COUNT = 5

function toHumidity(value) {
  if (value === null || value === undefined) return null
  const number = Number(value)
  return Number.isNaN(number) ? null : number
}

const dataPoints = computed(() =>
  [...props.readings]
    .map((reading) => ({ time: new Date(reading.time), humidity: toHumidity(reading.humidity) }))
    .filter((point) => !Number.isNaN(point.time.getTime()) && point.humidity !== null)
    .sort((a, b) => a.time - b.time),
)

const firstTime = computed(() => dataPoints.value[0]?.time ?? null)
const lastTime = computed(() => dataPoints.value[dataPoints.value.length - 1]?.time ?? null)

function xScale(time) {
  const start = firstTime.value
  const end = lastTime.value
  const span = start && end ? end.getTime() - start.getTime() : 0
  const ratio = span ? (time.getTime() - start.getTime()) / span : 0
  return PAD.left + ratio * (W - PAD.left - PAD.right)
}

function yScale(humidity) {
  const ratio = (humidity - Y_MIN) / (Y_MAX - Y_MIN)
  return PAD.top + (1 - ratio) * (H - PAD.top - PAD.bottom)
}

const linePath = computed(() => {
  const points = dataPoints.value.map((point) => [xScale(point.time), yScale(point.humidity)])
  if (points.length === 0) return null
  const first = points[0]
  const commands = [`M ${first[0]} ${first[1]}`]
  for (const point of points.slice(1)) {
    commands.push(`L ${point[0]} ${point[1]}`)
  }
  return commands.join(' ')
})

const areaPath = computed(() => {
  const points = dataPoints.value.map((point) => [xScale(point.time), yScale(point.humidity)])
  if (points.length < 2) return null
  const first = points[0]
  const last = points[points.length - 1]
  const bottom = H - PAD.bottom
  const commands = [`M ${first[0]} ${first[1]}`]
  for (const point of points.slice(1)) {
    commands.push(`L ${point[0]} ${point[1]}`)
  }
  commands.push(`L ${last[0]} ${bottom}`, `L ${first[0]} ${bottom}`, 'Z')
  return commands.join(' ')
})

const minHumidityValue = computed(() => toHumidity(props.minHumidity))
const maxHumidityValue = computed(() => toHumidity(props.maxHumidity))

const xTicks = computed(() => {
  const start = firstTime.value
  const end = lastTime.value
  if (!start || !end) return []
  const span = end.getTime() - start.getTime()
  const ticks = []
  for (let i = 0; i < X_TICK_COUNT; i++) {
    ticks.push(new Date(start.getTime() + (span * i) / (X_TICK_COUNT - 1)))
  }
  return ticks
})

const spanHours = computed(() => {
  const start = firstTime.value
  const end = lastTime.value
  if (!start || !end) return 0
  return (end.getTime() - start.getTime()) / (1000 * 60 * 60)
})

function formatTimeTick(date) {
  const timeOptions = { hour: '2-digit', minute: '2-digit', hourCycle: 'h23' }
  if (spanHours.value < 24) {
    return date.toLocaleTimeString([], timeOptions)
  }
  if (spanHours.value < 24 * 7) {
    return `${date.toLocaleDateString([], { month: 'short', day: 'numeric' })}, ${date.toLocaleTimeString([], timeOptions)}`
  }
  return date.toLocaleDateString([], { month: 'short', day: 'numeric', year: 'numeric' })
}

const singlePoint = computed(() => {
  if (dataPoints.value.length !== 1) return null
  const point = dataPoints.value[0]
  return { cx: xScale(point.time), cy: yScale(point.humidity) }
})
</script>

<template>
  <div class="humidity-graph">
    <svg
      class="humidity-graph-svg"
      viewBox="0 0 640 260"
      role="img"
      aria-label="Humidity readings over time"
    >
      <line
        v-for="humidity in Y_TICKS"
        :key="`gridline-${humidity}`"
        class="graph-grid-line"
        :x1="PAD.left"
        :x2="W - PAD.right"
        :y1="yScale(humidity)"
        :y2="yScale(humidity)"
      />
      <text
        v-for="humidity in Y_TICKS"
        :key="`y-tick-${humidity}`"
        class="graph-tick"
        :x="PAD.left - 8"
        :y="yScale(humidity) + 4"
        text-anchor="end"
      >
        {{ humidity }}%
      </text>

      <line
        class="graph-axis"
        :x1="PAD.left"
        :x2="W - PAD.right"
        :y1="H - PAD.bottom"
        :y2="H - PAD.bottom"
      />

      <line
        v-for="(tick, index) in xTicks"
        :key="`x-gridline-${index}`"
        class="graph-grid-line"
        :x1="xScale(tick)"
        :x2="xScale(tick)"
        :y1="PAD.top"
        :y2="H - PAD.bottom"
      />
      <text
        v-for="(tick, index) in xTicks"
        :key="`x-tick-${index}`"
        class="graph-tick"
        :x="xScale(tick)"
        :y="H - PAD.bottom + 18"
        text-anchor="middle"
      >
        {{ formatTimeTick(tick) }}
      </text>

      <line
        v-if="minHumidityValue !== null"
        class="graph-threshold graph-threshold-low"
        :x1="PAD.left"
        :x2="W - PAD.right"
        :y1="yScale(minHumidityValue)"
        :y2="yScale(minHumidityValue)"
      />
      <line
        v-if="maxHumidityValue !== null"
        class="graph-threshold graph-threshold-high"
        :x1="PAD.left"
        :x2="W - PAD.right"
        :y1="yScale(maxHumidityValue)"
        :y2="yScale(maxHumidityValue)"
      />

      <path v-if="areaPath" :d="areaPath" class="graph-area" />
      <path v-if="linePath" :d="linePath" class="graph-line" />

      <circle
        v-if="singlePoint"
        class="graph-point"
        :cx="singlePoint.cx"
        :cy="singlePoint.cy"
        r="4"
      />
    </svg>
  </div>
</template>

<style scoped>
.humidity-graph {
  background: var(--color-background-soft);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1.5rem;
}

.humidity-graph-svg {
  display: block;
  width: 100%;
  height: auto;
}

.graph-grid-line {
  stroke: var(--color-border);
  stroke-width: 1;
}

.graph-axis {
  stroke: var(--color-border-hover);
  stroke-width: 1.5;
}

.graph-tick {
  fill: var(--color-text);
  font-size: 11px;
  opacity: 0.75;
}

.graph-line {
  fill: none;
  stroke: hsla(160, 100%, 37%, 1);
  stroke-width: 2;
}

.graph-area {
  fill: hsla(160, 100%, 37%, 0.12);
}

.graph-point {
  fill: hsla(160, 100%, 37%, 1);
  stroke: var(--color-background);
  stroke-width: 2;
}

.graph-threshold {
  stroke-width: 1.5;
  stroke-dasharray: 5 4;
}

.graph-threshold-low {
  stroke: #f39c12;
}

.graph-threshold-high {
  stroke: #e74c3c;
}
</style>
