/** @type {import('tailwindcss').Config} */

export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}"
  ],
  theme: {
    extend: {
      colors: {
        lilac: {
          100: "#ede4fa",
          200: "#dac9f5",
          300: "#c8aef0",
          400: "#b593eb",
          500: "#a378e6",
          600: "#8260b8",
          700: "#62488a",
          800: "#41305c",
          900: "#21182e"
        },
      },

      fontFamily: {
        sans: ['Inter var'],
      },
    },
  },
  plugins: [],
}

