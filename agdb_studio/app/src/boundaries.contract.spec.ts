import { describe, expect, it } from "vitest";
// @ts-expect-error - lint boundary config is authored in mjs and has no TypeScript declaration.
import { boundaryConfig } from "../../eslint.boundaries.mjs";

type BoundaryEntry = {
  files: string[];
  rules: {
    "no-restricted-imports": [
      string,
      {
        patterns: Array<{
          group: string[];
          message: string;
        }>;
      },
    ];
  };
};

const getRuleByFileGlob = (glob: string): BoundaryEntry => {
  const entry = (boundaryConfig as BoundaryEntry[]).find((item) =>
    item.files.includes(glob),
  );

  expect(entry).toBeDefined();
  return entry as BoundaryEntry;
};

const getRestrictedGroups = (entry: BoundaryEntry): string[] => {
  const [, options] = entry.rules["no-restricted-imports"];
  return options.patterns.flatMap((pattern) => pattern.group);
};

describe("boundaryConfig contract", () => {
  it("keeps the expected number of boundary rule groups", () => {
    expect(boundaryConfig).toHaveLength(7);
  });

  it("prevents platform packages from importing shared and domain packages", () => {
    const platformRule = getRuleByFileGlob(
      "libs/core/{api,auth,router,design,utils}/src/**/*.{ts,vue,spec.ts}",
    );
    const restricted = getRestrictedGroups(platformRule);

    expect(restricted).toContain("@agdb-studio/common");
    expect(restricted).toContain("@agdb-studio/common/*");
    expect(restricted).toContain("@agdb-studio/notification");
    expect(restricted).toContain("@agdb-studio/notification/*");
    expect(restricted).toContain("@agdb-studio/profile");
    expect(restricted).toContain("@agdb-studio/profile/*");
    expect(restricted).toContain("@agdb-studio/db");
    expect(restricted).toContain("@agdb-studio/db/*");
    expect(restricted).toContain("@agdb-studio/query");
    expect(restricted).toContain("@agdb-studio/query/*");
    expect(restricted).toContain("@agdb-studio/user");
    expect(restricted).toContain("@agdb-studio/user/*");
    expect(restricted).toContain("@agdb-studio/cluster");
    expect(restricted).toContain("@agdb-studio/cluster/*");
  });

  it("prevents shared packages from importing domain packages", () => {
    const sharedRule = getRuleByFileGlob(
      "libs/features/{common,notification}/src/**/*.{ts,vue,spec.ts}",
    );
    const restricted = getRestrictedGroups(sharedRule);

    expect(restricted).toContain("@agdb-studio/profile");
    expect(restricted).toContain("@agdb-studio/profile/*");
    expect(restricted).toContain("@agdb-studio/db");
    expect(restricted).toContain("@agdb-studio/db/*");
    expect(restricted).toContain("@agdb-studio/query");
    expect(restricted).toContain("@agdb-studio/query/*");
    expect(restricted).toContain("@agdb-studio/user");
    expect(restricted).toContain("@agdb-studio/user/*");
    expect(restricted).toContain("@agdb-studio/cluster");
    expect(restricted).toContain("@agdb-studio/cluster/*");
  });

  it("prevents each domain package from importing other domains", () => {
    const domainGlobs = [
      ["libs/core/profile/src/**/*.{ts,vue,spec.ts}", "@agdb-studio/profile"],
      ["libs/features/db/src/**/*.{ts,vue,spec.ts}", "@agdb-studio/db"],
      ["libs/features/query/src/**/*.{ts,vue,spec.ts}", "@agdb-studio/query"],
      ["libs/features/user/src/**/*.{ts,vue,spec.ts}", "@agdb-studio/user"],
      [
        "libs/features/cluster/src/**/*.{ts,vue,spec.ts}",
        "@agdb-studio/cluster",
      ],
    ] as const;

    const allDomains = [
      "@agdb-studio/profile",
      "@agdb-studio/db",
      "@agdb-studio/query",
      "@agdb-studio/user",
      "@agdb-studio/cluster",
    ];

    for (const [glob, selfDomain] of domainGlobs) {
      const domainRule = getRuleByFileGlob(glob);
      const restricted = getRestrictedGroups(domainRule);

      for (const domain of allDomains) {
        if (domain === selfDomain) {
          expect(restricted).not.toContain(domain);
          expect(restricted).not.toContain(`${domain}/*`);
          continue;
        }

        expect(restricted).toContain(domain);
        expect(restricted).toContain(`${domain}/*`);
      }
    }
  });
});
