module.exports = [
  // ignore build artifacts and deps
  {
    ignores: ['dist/**', 'node_modules/**', 'src-tauri/target/**']
  },
  // apply rules to source files
  {
    files: ['src/**'],
    languageOptions: {
      parser: require('vue-eslint-parser'),
      parserOptions: {
        parser: require('@typescript-eslint/parser'),
        ecmaVersion: 2020,
        sourceType: 'module',
        extraFileExtensions: ['.vue']
      }
    },
    plugins: {
      vue: require('eslint-plugin-vue'),
      '@typescript-eslint': require('@typescript-eslint/eslint-plugin')
    },
    rules: {
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      'vue/html-self-closing': ['error', { html: { void: 'never', normal: 'always', component: 'always' } }]
    }
  }
];
