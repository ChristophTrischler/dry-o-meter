<script setup>
import { RouterLink, RouterView } from 'vue-router'

</script>

<template>
  <div id="app-layout">
    <header class="app-header">
      <div class="header-content">
        <RouterLink to="/" class="brand">
          <span class="brand-icon">💧</span>
          <span class="brand-text">Dry-O-Meter</span>
        </RouterLink>
        <nav>
          <RouterLink to="/" class="nav-link">Dashboard</RouterLink>
          <RouterLink to="/controller/new" class="nav-link btn-add">+ Add Controller</RouterLink>
        </nav>
      </div>
    </header>
    <main class="app-main">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app-header {
  background: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
  padding: 0 1.5rem;
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-content {
  max-width: 1200px;
  margin: 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 56px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  text-decoration: none;
  color: var(--color-heading);
  font-weight: 600;
  font-size: 1.15rem;
}

.brand-icon {
  font-size: 1.4rem;
}

nav {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.nav-link {
  text-decoration: none;
  color: var(--color-text);
  padding: 0.4rem 0.8rem;
  border-radius: 6px;
  font-size: 0.9rem;
  transition: background-color 0.2s;
}

.nav-link:hover {
  background-color: var(--color-border);
}

.nav-link.router-link-exact-active {
  color: var(--color-heading);
  font-weight: 500;
  background-color: var(--color-border);
}

.btn-add {
  background: hsla(160, 100%, 37%, 1);
  color: white;
  font-weight: 500;
}

.btn-add:hover {
  background: hsla(160, 100%, 32%, 1);
}

.app-main {
  max-width: 1200px;
  margin: 0 auto;
  padding: 1.5rem;
}
</style>
