import { createApp } from "vue";
import App from "./App.vue";
import "./styles/styles.css";

// Disable default browser context menu
document.addEventListener('contextmenu', event => {
  event.preventDefault();
}, false);

// Apply theme based on system preference or stored preference
const initTheme = () => {
  const storedTheme = localStorage.getItem('theme');
  
  if (storedTheme) {
    document.documentElement.classList.add(storedTheme === 'dark' ? 'dark-mode' : 'light-mode');
  } else {
    // Use system preference
    if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.classList.add('dark-mode');
    } else {
      document.documentElement.classList.add('light-mode');
    }
  }
};

// Initialize theme
initTheme();

createApp(App).mount("#app");
