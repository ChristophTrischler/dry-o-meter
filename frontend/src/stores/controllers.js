import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import * as api from '@/api/controllers'

export const useControllersStore = defineStore('controllers', () => {
  const controllers = ref([])
  const currentController = ref(null)
  const loading = ref(false)
  const error = ref(null)

  async function fetchControllers() {
    loading.value = true
    error.value = null
    try {
      controllers.value = await api.getControllers()
    } catch (e) {
      error.value = e.message
    } finally {
      loading.value = false
    }
  }

  async function fetchController(id) {
    loading.value = true
    error.value = null
    try {
      currentController.value = await api.getController(id)
    } catch (e) {
      error.value = e.message
    } finally {
      loading.value = false
    }
  }

  async function addController(data) {
    loading.value = true
    error.value = null
    try {
      const result = await api.createController(data)
      await fetchControllers()
      return result
    } catch (e) {
      error.value = e.message
      throw e
    } finally {
      loading.value = false
    }
  }

  async function editController(id, data) {
    loading.value = true
    error.value = null
    try {
      const updated = await api.updateController(id, data)
      currentController.value = updated
      await fetchControllers()
      return updated
    } catch (e) {
      error.value = e.message
      throw e
    } finally {
      loading.value = false
    }
  }

  async function removeController(id) {
    loading.value = true
    error.value = null
    try {
      await api.deleteController(id)
      await fetchControllers()
    } catch (e) {
      error.value = e.message
      throw e
    } finally {
      loading.value = false
    }
  }

  async function fetchControllerWithReadings(id, limit = null) {
    loading.value = true
    error.value = null
    try {
      currentController.value = await api.getControllerWithReadings(id, limit)
    } catch (e) {
      error.value = e.message
    } finally {
      loading.value = false
    }
  }

  async function fetchRecentReadings(id, hours = null) {
    loading.value = true
    error.value = null
    try {
      currentController.value = await api.getRecentReadings(id, hours)
    } catch (e) {
      error.value = e.message
    } finally {
      loading.value = false
    }
  }

  const controllerCount = computed(() => controllers.value.length)

  return {
    controllers,
    currentController,
    loading,
    error,
    controllerCount,
    fetchControllers,
    fetchController,
    addController,
    editController,
    removeController,
    fetchControllerWithReadings,
    fetchRecentReadings,
  }
})
