import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./templates/*.html"],
  theme: {
    extend: {
      fontFamily: {
        title: ['Designer'],
        goose: ['Cocogoose'],
        bebas: ['Bebas'],
        biko: ['Biko']
      }
    }
  },
  plugins: [],
}

export default config;

