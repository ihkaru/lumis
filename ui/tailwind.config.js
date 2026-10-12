/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{svelte,js,ts}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Google Sans"', '"Google Sans Text"', 'sans-serif'],
        mono: ['"Google Sans Code"', 'monospace'],
      },
    },
  },
  plugins: [],
};
