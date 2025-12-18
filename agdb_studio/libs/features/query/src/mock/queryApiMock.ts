export const queryApiMock = {
  "": {
    followers: ["select", "insert", "update", "delete"],
  },
  select: {
    followers: ["key_count", "search", "from", "limit", "values"],
  },
  values: {
    followers: ["key_count", "search", "from", "limit", "values"],
    values: ["string", "number", "boolean"],
  },
  key_count: {
    followers: ["search", "from", "limit", "values"],
  },
  search: {
    followers: ["from", "limit", "values"],
  },
  from: {
    followers: ["limit", "values"],
    values: ["number"],
  },
  limit: {
    followers: [],
    values: ["number"],
  },
  insert: {
    followers: ["values"],
  },
  update: {
    followers: ["search", "values"],
  },
  delete: {
    followers: ["search"],
  },
};
