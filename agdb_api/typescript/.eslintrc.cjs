module.exports = {
    root: true,
    extends: [
        "eslint:recommended",
    ],
    env: {
        node: true,
    },
    ignorePatterns: ["src/openapi.d.ts", ".gitignore", "dist/"],
    parser: "@typescript-eslint/parser",
    parserOptions: {
        ecmaVersion: "latest"
    },
};
