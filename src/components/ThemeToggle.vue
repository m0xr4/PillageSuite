<template>
  <button 
    class="theme-toggle" 
    @click="toggleTheme" 
    aria-label="Toggle dark/light mode"
  >
    <!-- Sun icon for dark mode -->
    <svg 
      v-if="isDarkMode" 
      xmlns="http://www.w3.org/2000/svg" 
      class="icon" 
      viewBox="0 0 20 20" 
      fill="currentColor"
    >
      <path 
        fill-rule="evenodd" 
        d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" 
        clip-rule="evenodd" 
      />
    </svg>
    <!-- Moon icon for light mode -->
    <svg 
      v-else 
      xmlns="http://www.w3.org/2000/svg" 
      class="icon" 
      viewBox="0 0 20 20" 
      fill="currentColor"
    >
      <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
    </svg>
  </button>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue';

// Theme state
const isDarkMode = ref(false);

// Check system preference and localStorage on mount
onMounted(() => {
  // Check if theme is stored in localStorage
  const storedTheme = localStorage.getItem('theme');
  
  if (storedTheme) {
    // Use stored preference
    isDarkMode.value = storedTheme === 'dark';
  } else {
    // Use system preference
    isDarkMode.value = window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  
  // Apply theme
  applyTheme();
  
  // Listen for system preference changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    if (!localStorage.getItem('theme')) {
      isDarkMode.value = e.matches;
      applyTheme();
    }
  });
});

// Watch for theme changes
watch(isDarkMode, () => {
  applyTheme();
});

// Toggle theme function
const toggleTheme = () => {
  isDarkMode.value = !isDarkMode.value;
  // Save preference to localStorage
  localStorage.setItem('theme', isDarkMode.value ? 'dark' : 'light');
};

// Apply theme function
const applyTheme = () => {
  if (isDarkMode.value) {
    document.documentElement.classList.add('dark-mode');
    document.documentElement.classList.remove('light-mode');
  } else {
    document.documentElement.classList.add('light-mode');
    document.documentElement.classList.remove('dark-mode');
  }
};
</script>

<style scoped>
.theme-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-dropdown-bg);
  border: none;
  padding: 0.5rem;
  color: var(--color-text-light);
  cursor: pointer;
  border-radius: 0.375rem;
  transition: background-color 0.2s, transform 0.1s;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  height: 38px;
  width: 38px;
}

.theme-toggle:hover {
  background-color: var(--color-dropdown-hover);
  transform: translateY(-1px);
}

.theme-toggle:active {
  transform: translateY(0);
}

.icon {
  height: 1.25rem;
  width: 1.25rem;
  transition: transform 0.3s ease;
}

.theme-toggle:hover .icon {
  transform: rotate(12deg);
}

/* Dark mode specific styles */
:global(.dark-mode) .theme-toggle {
  color: var(--color-text-light);
}

/* Light mode specific styles */
:global(.light-mode) .theme-toggle {
  color: var(--color-text-light);
}
</style> 