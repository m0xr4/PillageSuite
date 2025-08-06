<template>
  <div class="layout-container">
    <!-- Header -->
    <header class="app-header">
      <div class="flex">
        <div class="app-branding">
          <div class="logo-container">
            <svg class="app-logo" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <!-- Folder -->
              <path d="M22 5h-9l-2-2H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h19c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2z" transform="scale(0.92) translate(1, 1)" fill="currentColor"/>
              <!-- Share Icon based on svgrepo reference -->
              <g transform="translate(6.5, 7) scale(0.025)" fill="#1e293b" fill-opacity="0.8">
                <path d="M339.588,314.529c-14.215,0-27.456,4.133-38.621,11.239l-112.682-78.67c1.809-6.315,2.798-12.976,2.798-19.871    c0-6.896-0.989-13.557-2.798-19.871l109.64-76.547c11.764,8.356,26.133,13.286,41.662,13.286c39.79,0,72.047-32.257,72.047-72.047    C411.634,32.258,379.378,0,339.588,0c-39.79,0-72.047,32.257-72.047,72.047c0,5.255,0.578,10.373,1.646,15.308l-112.424,78.491    c-10.974-6.759-23.892-10.666-37.727-10.666c-39.79,0-72.047,32.257-72.047,72.047s32.256,72.047,72.047,72.047    c13.834,0,26.753-3.907,37.727-10.666l113.292,79.097c-1.629,6.017-2.514,12.34-2.514,18.872c0,39.79,32.257,72.047,72.047,72.047    c39.79,0,72.047-32.257,72.047-72.047C411.635,346.787,379.378,314.529,339.588,314.529z"/>
              </g>
            </svg>
          </div>
          <h1 class="app-title">Pillage Suite</h1>
        </div>
        <div class="flex header-controls">
          <NavDropdown 
            :isConnected="isConnected" 
            @tabChange="handleTabChange"
          />
          <div class="controls-group">
            <ThemeToggle />
            <div class="connection-status" :class="{ 'connected': isConnected, 'disconnected': !isConnected }">
              <svg v-if="isConnected" xmlns="http://www.w3.org/2000/svg" class="status-icon" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" class="status-icon" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
              <span>{{ isConnected ? 'Connected' : 'Disconnected' }}</span>
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Content with Sidebar -->
    <main class="main-content">
      <!-- Sidebar Navigation -->
      <aside class="sidebar">
        <slot name="sidebar"></slot>
      </aside>

      <!-- Main Content Area -->
      <div class="content-area">
        <slot name="main"></slot>
      </div>
    </main>
  </div>
</template>

<script setup>
import NavDropdown from './NavDropdown.vue';
import ThemeToggle from './ThemeToggle.vue';


const props = defineProps({
  isConnected: Boolean
});

const emit = defineEmits(['tabChange']);

// Handle tab change from dropdown
const handleTabChange = (tabIndex) => {
  emit('tabChange', tabIndex);
};
</script>

<style scoped>
/* Ensure the layout takes full height */
:deep(.space-y-6 > *) {
  margin-bottom: 1.5rem;
}

:deep(.space-y-4 > *) {
  margin-bottom: 1rem;
}

.layout-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden; /* Prevent scrolling on the container */
  box-sizing: border-box;
}

.app-header {
  background: linear-gradient(to right, var(--color-primary), var(--color-primary-darker, var(--color-primary)));
  color: var(--color-text-light);
  padding: 0.75rem 1.5rem;
  box-shadow: 0 2px 10px var(--color-card-shadow);
  position: sticky;
  top: 0;
  left: 0;
  right: 0;
  z-index: 100;
  height: 60px; /* Fixed height for the header */
  box-sizing: border-box;
  display: flex;
  align-items: center;
}

.app-header::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image: radial-gradient(circle at top right, rgba(255, 255, 255, 0.15) 0%, transparent 60%);
  z-index: -1;
}

.app-branding {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.logo-container {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 2.25rem;
  width: 2.25rem;
  border-radius: 0.5rem;
  background-color: rgba(255, 255, 255, 0.15);
  padding: 0.375rem;
}

.app-logo {
  height: 1.5rem;
  width: 1.5rem;
  color: var(--color-text-light);
}

.app-title {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0;
  letter-spacing: -0.02em;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
  height: calc(100vh - 60px); /* Adjust height based on header */
  position: relative;
  box-sizing: border-box;
}

.sidebar {
  width: 375px;
  background-color: var(--color-sidebar-bg);
  border-right: 1px solid var(--color-border);
  overflow-y: auto;
  height: 100%; /* Full height of the main content area */
  box-sizing: border-box;
}

.content-area {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
  background-color: var(--color-content-bg);
  height: 100%; /* Full height of the main content area */
  box-sizing: border-box;
}

.flex {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.header-controls {
  display: flex;
  justify-content: flex-end;
  align-items: center;
}

.header-controls > *:not(:last-child) {
  margin-right: 0.75rem;
}

.controls-group {
  display: flex;
  align-items: center;
}

.connection-status {
  display: flex;
  align-items: center;
  background-color: var(--color-dropdown-bg);
  border-radius: 0.375rem;
  padding: 0.5rem 1rem;
  color: var(--color-text-light);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  font-weight: 500;
  margin-left: 4px;
}

.connection-status.connected {
  color: var(--color-success);
}

.connection-status.disconnected {
  color: var(--color-error);
}

.status-icon {
  height: 1.25rem;
  width: 1.25rem;
  margin-right: 0.25rem;
}
</style> 