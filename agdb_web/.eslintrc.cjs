module.exports = {
    root: true,
    env: {
        node: true,
        browser: true,
    },
    extends: [
        "plugin:vue/vue3-recommended",
        "@nuxtjs/eslint-config-typescript",
        "eslint-config-prettier",
    ],
    rules: {
        "no-console": process.env.NODE_ENV === "production" ? "error" : "off",
        "no-debugger": process.env.NODE_ENV === "production" ? "error" : "off",
        indent: "off",
        "vue/no-multiple-template-root": "off",
    },
    globals: {
        $nuxt: true,
    },
    parserOptions: {
        extraFileExtensions: [".vue"],
    },
};
