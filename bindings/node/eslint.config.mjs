import js from '@eslint/js';
import globals from 'globals';

export default [
  {
    ignores: [
      'target/**',
      '**/node_modules/**',
      '**/pkg-npm/**',
      '**/artifacts/**',
      '**/release/**',
      '**/npm/**',
    ],
  },
  js.configs.recommended,
  {
    files: ['**/*.{js,mjs,cjs}'],
    languageOptions: { globals: globals.node },
    rules: {
      'no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      eqeqeq: 'error',
      'no-var': 'error',
      'prefer-const': 'error',
      curly: ['error', 'all'],
      'max-statements-per-line': ['error', { max: 1 }],
    },
  },
];
