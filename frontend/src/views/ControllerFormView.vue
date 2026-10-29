<script setup>
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useControllersStore } from '@/stores/controllers'

const props = defineProps({
  id: { type: String, default: null },
})

const router = useRouter()
const store = useControllersStore()

const form = ref({
  name: '',
  location: '',
  min_humidity: '',
  max_humidity: '',
})

const isEdit = computed(() => props.id != null)
const pageTitle = computed(() => (isEdit.value ? 'Edit Controller' : 'Add Controller'))
const submitLabel = computed(() => (isEdit.value ? 'Save Changes' : 'Create Controller'))

onMounted(async () => {
  if (isEdit.value) {
    await store.fetchController(Number(props.id))
    if (store.currentController) {
      form.value = {
        name: store.currentController.name,
        location: store.currentController.location,
        min_humidity: store.currentController.min_humidity ?? '',
        max_humidity: store.currentController.max_humidity ?? '',
      }
    }
  }
})

async function handleSubmit() {
  const payload = {
    name: form.value.name,
    location: form.value.location,
    min_humidity: form.value.min_humidity !== '' ? String(form.value.min_humidity) : null,
    max_humidity: form.value.max_humidity !== '' ? String(form.value.max_humidity) : null,
  }

  try {
    if (isEdit.value) {
      await store.editController(Number(props.id), payload)
      router.push(`/controller/${props.id}`)
    } else {
      const result = await store.addController(payload)
      router.push(`/controller/${result.controller_id}`)
    }
  } catch (e) {
    alert('Error: ' + e.message)
  }
}
</script>

<template>
  <div class="form-view">
    <RouterLink to="/" class="back-link">&larr; Back</RouterLink>
    <h1>{{ pageTitle }}</h1>

    <form @submit.prevent="handleSubmit" class="controller-form">
      <div class="form-group">
        <label for="name">Name *</label>
        <input
          id="name"
          v-model="form.name"
          type="text"
          required
          maxlength="100"
          placeholder="e.g. Living Room Sensor"
        />
      </div>

      <div class="form-group">
        <label for="location">Location *</label>
        <input
          id="location"
          v-model="form.location"
          type="text"
          required
          maxlength="100"
          placeholder="e.g. Main Floor"
        />
      </div>

      <div class="form-row">
        <div class="form-group">
          <label for="min-humidity">Min Humidity %</label>
          <input
            id="min-humidity"
            v-model="form.min_humidity"
            type="number"
            step="0.01"
            min="0"
            max="100"
            placeholder="e.g. 30.00"
          />
        </div>

        <div class="form-group">
          <label for="max-humidity">Max Humidity %</label>
          <input
            id="max-humidity"
            v-model="form.max_humidity"
            type="number"
            step="0.01"
            min="0"
            max="100"
            placeholder="e.g. 60.00"
          />
        </div>
      </div>

      <div class="form-actions">
        <RouterLink :to="isEdit ? `/controller/${id}` : '/'" class="btn btn-secondary">
          Cancel
        </RouterLink>
        <button type="submit" class="btn btn-primary" :disabled="store.loading">
          {{ store.loading ? 'Saving...' : submitLabel }}
        </button>
      </div>
    </form>
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

h1 {
  font-size: 1.6rem;
  color: var(--color-heading);
  margin-bottom: 1.5rem;
}

.controller-form {
  max-width: 560px;
  background: var(--color-background-soft);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 1.5rem;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  font-size: 0.85rem;
  font-weight: 500;
  margin-bottom: 0.35rem;
  color: var(--color-heading);
}

.form-group input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 0.9rem;
  background: var(--color-background);
  color: var(--color-text);
  transition: border-color 0.2s;
}

.form-group input:focus {
  outline: none;
  border-color: hsla(160, 100%, 37%, 1);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 1.5rem;
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

.btn-primary {
  background: hsla(160, 100%, 37%, 1);
  color: white;
}

.btn-primary:hover {
  background: hsla(160, 100%, 32%, 1);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--color-background-mute);
  color: var(--color-text);
}

.btn-secondary:hover {
  background: var(--color-border);
}
</style>
