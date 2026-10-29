const BASE_URL = '/api/controllers'

async function request(url, options = {}) {
  const res = await fetch(url, {
    headers: { 'Content-Type': 'application/json', ...options.headers },
    ...options,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => null)
    const message = body?.brief || body?.detail || `Request failed: ${res.status}`
    throw new Error(message)
  }
  if (res.status === 204 || res.headers.get('content-length') === '0') return null
  return res.json()
}

export function getControllers() {
  return request(BASE_URL)
}

export function getController(id) {
  return request(`${BASE_URL}/${id}`)
}

export function createController(data) {
  return request(BASE_URL, { method: 'POST', body: JSON.stringify(data) })
}

export function updateController(id, data) {
  return request(`${BASE_URL}/${id}`, { method: 'PUT', body: JSON.stringify(data) })
}

export function deleteController(id) {
  return request(`${BASE_URL}/${id}`, { method: 'DELETE' })
}

export function reclaimControllerUuid(id) {
  return request(`${BASE_URL}/${id}/reclaim-uuid`, { method: 'POST' })
}

export function getControllerWithReadings(id, limit = null) {
  const params = limit != null ? `?limit=${limit}` : ''
  return request(`${BASE_URL}/${id}/readings${params}`)
}

export function getRecentReadings(id, hours = null) {
  const params = hours != null ? `?hours=${hours}` : ''
  return request(`${BASE_URL}/${id}/readings/recent${params}`)
}
