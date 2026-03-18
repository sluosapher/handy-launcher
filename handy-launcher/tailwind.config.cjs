/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: '#1c1c1e',
        card: '#2c2c2e',
        surface: '#f7f7f7',
        accent: '#0a84ff',
        success: '#34c759',
        warning: '#ff9500',
        error: '#ff3b30'
      }
    }
  },
  plugins: []
};
