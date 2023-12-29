require("@rushstack/eslint-patch/modern-module-resolution");

module.exports = {
    root: true,
    extends: [
        "eslint:recommended",
    ],
    parserOptions: {
        ecmaVersion: "latest",
    },
};
