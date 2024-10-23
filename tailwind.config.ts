import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./templates/*.html"],
  theme: {
    extend: {
      fontFamily: {
        title: ['Designer'],
        goose: ['Cocogoose']
      }
    }
  },
  plugins: [],
}

export default config;

