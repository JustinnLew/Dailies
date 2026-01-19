import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import { defineConfig } from 'eslint/config';

export default defineConfig(
  { 
    ignores: ["dist", "node_modules", "build"] 
  },
  {
    // This targets your files
    files: ["**/*.{js,ts,tsx}"],
    // This combines the recommended rules into this block
    extends: [
      eslint.configs.recommended,
      ...tseslint.configs.recommended,
    ],
    languageOptions: {
      parserOptions: {
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    rules: {
      // Manually add this to verify it's working
      "no-unused-vars": "warn",
      "@typescript-eslint/no-unused-vars": "warn",
    },
  }
);