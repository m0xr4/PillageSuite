<template>
  <div class="nav-dropdown">
    <!-- Dropdown Button -->
    <button 
      class="dropdown-button" 
      @click="isOpen = !isOpen"
    >
      <div class="menu-label">
        <span>Menu</span>
      </div>
      <svg xmlns="http://www.w3.org/2000/svg" class="dropdown-icon" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
      </svg>
    </button>
    
    <!-- Dropdown Menu -->
    <div v-show="isOpen" class="dropdown-content">
      <div 
        v-for="(tab, index) in tabs" 
        :key="index"
        class="dropdown-item" 
        :class="{ 'active': activeTab === index }"
        @click="selectTab(index)"
      >
        <div class="tab-icon">
          <svg v-if="index === 0" xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.05 3.636a1 1 0 010 1.414 7 7 0 000 9.9 1 1 0 11-1.414 1.414 9 9 0 010-12.728 1 1 0 011.414 0zm9.9 0a1 1 0 011.414 0 9 9 0 010 12.728 1 1 0 11-1.414-1.414 7 7 0 000-9.9 1 1 0 010-1.414zM7.879 6.464a1 1 0 010 1.414 3 3 0 000 4.243 1 1 0 11-1.415 1.414 5 5 0 010-7.07 1 1 0 011.415 0zm4.242 0a1 1 0 011.415 0 5 5 0 010 7.072 1 1 0 01-1.415-1.415 3 3 0 000-4.242 1 1 0 010-1.415z" clip-rule="evenodd" />
          </svg>
          <svg v-if="index === 1" xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
            <path d="M9 9a2 2 0 114 0 2 2 0 01-4 0z" />
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-13a4 4 0 00-3.446 6.032l-2.261 2.26a1 1 0 101.414 1.415l2.261-2.261A4 4 0 1011 5z" clip-rule="evenodd" />
          </svg>
          <svg v-if="index === 2" xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM6.293 6.707a1 1 0 010-1.414l3-3a1 1 0 011.414 0l3 3a1 1 0 01-1.414 1.414L11 5.414V13a1 1 0 11-2 0V5.414L7.707 6.707a1 1 0 01-1.414 0z" clip-rule="evenodd" />
          </svg>
          <svg v-if="index === 3" xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
            <path d="M2 10a8 8 0 018-8v8h8a8 8 0 11-16 0z" />
            <path d="M12 2.252A8.014 8.014 0 0117.748 8H12V2.252z" />
          </svg>
          <svg v-if="index === 4" xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 6a1 1 0 011-1h6a1 1 0 110 2H7a1 1 0 01-1-1zm1 3a1 1 0 100 2h6a1 1 0 100-2H7z" clip-rule="evenodd" />
          </svg>
          <svg v-if="index === 5" xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
          </svg>
        </div>
        <span class="tab-label">{{ tab.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, onUnmounted } from 'vue';

const props = defineProps({
  isConnected: Boolean
});

const emit = defineEmits(['tabChange']);

// Define tabs
const tabs = [
  { label: 'Connection' },
  { label: 'Discovery' },
  { label: 'Import' },
  { label: 'Stats' },
  { label: 'Active Index' },
  { label: 'About' }
];

// Dropdown state
const isOpen = ref(false);

// Active tab state
const activeTab = ref(0);

// Close dropdown when clicking outside
const handleClickOutside = (event) => {
  const dropdown = event.target.closest('.nav-dropdown');
  if (!dropdown) {
    isOpen.value = false;
  }
};

// Add click outside listener
document.addEventListener('click', handleClickOutside);

// Select tab function
const selectTab = (index) => {
  activeTab.value = index;
  isOpen.value = false;
  emit('tabChange', index);
};

// Clean up event listener
onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped>
.nav-dropdown {
  position: relative;
  display: inline-block;
  background-color: var(--color-dropdown-bg);
  border-radius: 0.375rem;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

.dropdown-button {
  display: flex;
  align-items: center;
  background-color: transparent;
  border: none;
  padding: 0.5rem 1rem;
  color: var(--color-text-light);
  cursor: pointer;
  border-radius: 0.375rem;
  transition: background-color 0.2s, transform 0.1s;
  font-weight: 500;
  min-height: 38px;
}

.dropdown-button:hover {
  background-color: var(--color-dropdown-hover);
  transform: translateY(-1px);
}

.dropdown-button:active {
  transform: translateY(0);
}

.menu-label {
  display: flex;
  align-items: center;
}

.icon {
  height: 1.25rem;
  width: 1.25rem;
  margin-right: 0.25rem;
}

.dropdown-icon {
  height: 1rem;
  width: 1rem;
  margin-left: 0.5rem;
  transition: transform 0.2s ease;
}

.dropdown-button:hover .dropdown-icon {
  transform: translateY(1px);
}

.dropdown-content {
  position: absolute;
  right: 0;
  top: calc(100% + 0.25rem);
  min-width: 220px;
  background-color: var(--color-dropdown-menu-bg);
  border-radius: 0.5rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  overflow: hidden;
  animation: dropdown-appear 0.2s ease forwards;
  transform-origin: top right;
}

@keyframes dropdown-appear {
  from {
    opacity: 0;
    transform: scale(0.98);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.dropdown-item {
  display: flex;
  align-items: center;
  padding: 0.75rem 1rem;
  color: var(--color-text);
  cursor: pointer;
  transition: background-color 0.2s, color 0.2s;
  border-left: 2px solid transparent;
}

.dropdown-item:hover {
  background-color: var(--color-dropdown-item-hover);
  color: var(--color-primary);
}

.dropdown-item.active {
  background-color: var(--color-dropdown-item-active-bg);
  color: var(--color-primary);
  border-left: 2px solid var(--color-primary);
}

.tab-icon {
  display: flex;
  align-items: center;
  margin-right: 0.75rem;
  color: currentColor;
  opacity: 0.8;
}

.tab-label {
  font-size: 0.9rem;
  font-weight: 500;
}
</style> 