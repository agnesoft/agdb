const platformPackages = [
  "@agdb-studio/api",
  "@agdb-studio/auth",
  "@agdb-studio/router",
  "@agdb-studio/design",
  "@agdb-studio/utils",
];

const sharedPackages = ["@agdb-studio/common", "@agdb-studio/notification"];

const domainPackages = [
  "@agdb-studio/profile",
  "@agdb-studio/db",
  "@agdb-studio/query",
  "@agdb-studio/user",
  "@agdb-studio/cluster",
];

const withWildcards = (packages) =>
  packages.flatMap((packageName) => [packageName, `${packageName}/*`]);

const toPatternObjects = (packages, message) =>
  withWildcards(packages).map((group) => ({ group: [group], message }));

const createRule = (patterns) => [
  "error",
  {
    patterns,
  },
];

const createDomainRule = (packageName, files, allow = []) => {
  const forbiddenDomains = domainPackages.filter(
    (name) => name !== packageName && !allow.includes(name),
  );

  return {
    files,
    rules: {
      "no-restricted-imports": createRule(
        toPatternObjects(
          forbiddenDomains,
          "Domain packages must not import other domain packages directly.",
        ),
      ),
    },
  };
};

const boundaryConfig = [
  {
    files: [
      "libs/core/{api,auth,router,design,utils}/src/**/*.{ts,vue,spec.ts}",
    ],
    rules: {
      "no-restricted-imports": createRule(
        toPatternObjects(
          [...sharedPackages, ...domainPackages],
          "Platform packages must not depend on shared or domain packages.",
        ),
      ),
    },
  },
  {
    files: ["libs/features/{common,notification}/src/**/*.{ts,vue,spec.ts}"],
    rules: {
      "no-restricted-imports": createRule(
        toPatternObjects(
          domainPackages,
          "Shared packages must not depend on domain packages.",
        ),
      ),
    },
  },
  createDomainRule("@agdb-studio/profile", [
    "libs/core/profile/src/**/*.{ts,vue,spec.ts}",
  ]),
  createDomainRule("@agdb-studio/db", [
    "libs/features/db/src/**/*.{ts,vue,spec.ts}",
  ]),
  createDomainRule("@agdb-studio/query", [
    "libs/features/query/src/**/*.{ts,vue,spec.ts}",
  ]),
  createDomainRule("@agdb-studio/user", [
    "libs/features/user/src/**/*.{ts,vue,spec.ts}",
  ]),
  createDomainRule("@agdb-studio/cluster", [
    "libs/features/cluster/src/**/*.{ts,vue,spec.ts}",
  ]),
];

export { boundaryConfig };
