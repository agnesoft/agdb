// @ts-check
import withNuxt from "./.nuxt/eslint.config.mjs";

export default withNuxt({
  rules: {
    "@stylistic/quotes": ["error", "double"],
    "@stylistic/comma-dangle": ["error", "always-multiline"],
    "@stylistic/semi": ["error", "always"],
    "vue/singleline-html-element-content-newline": "off",
    "vue/max-attributes-per-line": "off",
    "vue/comma-dangle": ["error", "always-multiline"],
    "@stylistic/quote-props": ["error", "as-needed"],
    "vue/html-self-closing": [
      "error",
      {
        html: {
          void: "always",
          normal: "never",
          component: "always",
        },
        svg: "always",
        math: "always",
      },
    ],
  },
});
