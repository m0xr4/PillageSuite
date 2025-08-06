<template>
  <div v-if="show" class="warning-dialog-container" :class="{ 'overlay-mode': overlayMode }">
    <div class="warning-dialog-backdrop" aria-hidden="true"></div>
    
    <div class="warning-dialog-panel">
      <div class="warning-dialog-content">
        <div class="warning-dialog-header">
          <div class="warning-icon">
            <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div class="warning-text">
            <h3 class="warning-title" id="modal-title">
              {{ title }}
            </h3>
            <div class="warning-message">
              <p>{{ message }}</p>
            </div>
          </div>
        </div>
        <div class="warning-dialog-actions">
          <button 
            v-if="showLimitButton"
            type="button" 
            class="warning-limit-button"
            @click="$emit('limit')"
          >
            {{ limitText }}
          </button>
          <button 
            type="button" 
            class="warning-confirm-button"
            @click="$emit('confirm')"
          >
            {{ confirmText }}
          </button>
          <button 
            type="button" 
            class="warning-cancel-button"
            @click="$emit('cancel')"
          >
            {{ cancelText }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  show: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: 'Warning'
  },
  message: {
    type: String,
    required: true
  },
  confirmText: {
    type: String,
    default: 'Proceed'
  },
  cancelText: {
    type: String,
    default: 'Cancel'
  },
  limitText: {
    type: String,
    default: 'Draw Limited'
  },
  showLimitButton: {
    type: Boolean,
    default: true
  },
  overlayMode: {
    type: Boolean,
    default: false
  }
});

defineEmits(['confirm', 'cancel', 'limit']);
</script>

<style scoped>
.warning-dialog-container {
  position: fixed;
  inset: 0;
  z-index: 50;
  overflow-y: auto;
  display: flex;
  align-items: center;
  justify-content: center;
}

.warning-dialog-container.overlay-mode {
  position: absolute;
}

.warning-dialog-backdrop {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  transition: opacity 0.3s ease;
}

.overlay-mode .warning-dialog-backdrop {
  position: absolute;
}

.warning-dialog-panel {
  position: relative;
  width: 100%;
  max-width: 28rem;
  background-color: var(--color-card-bg);
  border-radius: 0.5rem;
  overflow: hidden;
  box-shadow: 0 10px 15px -3px var(--color-card-shadow), 0 4px 6px -2px var(--color-card-shadow);
  transform: translateY(0);
  transition: transform 0.3s ease;
  border: 1px solid var(--color-border);
}

.warning-dialog-content {
  padding: 1.5rem;
}

.warning-dialog-header {
  display: flex;
  align-items: flex-start;
}

.warning-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 9999px;
  background-color: rgba(var(--color-error-rgb, 239, 68, 68), 0.15);
  margin-right: 1rem;
  color: var(--color-error);
}

.warning-text {
  flex: 1;
}

.warning-title {
  font-size: 1.125rem;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: 0.5rem;
}

.warning-message {
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.warning-dialog-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 1.5rem;
  gap: 0.75rem;
}

.warning-confirm-button {
  padding: 0.5rem 1rem;
  background-color: var(--color-error);
  color: white;
  font-weight: 500;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  transition: background-color 0.2s;
  border: none;
}

.warning-confirm-button:hover {
  background-color: var(--color-error-darker, var(--color-error));
  filter: brightness(90%);
}

.warning-cancel-button {
  padding: 0.5rem 1rem;
  background-color: var(--color-button-bg);
  color: var(--color-text);
  font-weight: 500;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font-size: 0.875rem;
  transition: background-color 0.2s;
}

.warning-cancel-button:hover {
  background-color: var(--color-button-hover);
}

.warning-limit-button {
  padding: 0.5rem 1rem;
  background-color: var(--color-primary);
  color: white;
  font-weight: 500;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  transition: background-color 0.2s;
  border: none;
}

.warning-limit-button:hover {
  background-color: var(--color-primary-darker);
}
</style> 