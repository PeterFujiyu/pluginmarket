/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
    "./public/index.html"
  ],
  theme: {
    extend: {
      colors: {
        'claude-orange': '#FF8C47',
        'claude-bg': '#F9F9F8',
        'claude-text': '#2F2F2F',
        'claude-light': '#FEFEFE',
      },
      fontFamily: {
        'claude': ['system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
      }
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
  ],
}