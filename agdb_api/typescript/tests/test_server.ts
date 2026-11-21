import { AgdbApi } from "../src/api.ts";
import { FetchClient } from "../src/fetch_client.ts";
import { expect } from "vitest";

/**
 * The `test_server` module of the agdb_server tests the route tests are
 * generated from: the server and the cluster `agdb_harness` starts for the
 * whole run, named in its environment variables.
 */

let counter = 1;

function env(name: string): string {
    const value = process.env[name];

    if (value === undefined || value === "") {
        throw new Error(`${name} is not set: run the tests with agdb_harness`);
    }

    return value;
}

/** The name of the running test, the user agent of its clients. */
export function test_agent_name(): string {
    return expect.getState().currentTestName ?? "agdb_api_test";
}

export function api_with_agent(
    address: string,
    agent: string,
): AgdbApi<FetchClient> {
    return AgdbApi.new(new FetchClient(agent), address);
}

export function api_for_test(address: string): AgdbApi<FetchClient> {
    return api_with_agent(address, test_agent_name());
}

export function next_user_name(): string {
    return `db_user${process.pid}_${counter++}`;
}

export function next_db_name(): string {
    return `db${process.pid}_${counter++}`;
}

export class TestServer {
    constructor(
        public data_dir: string,
        public api: AgdbApi<FetchClient>,
    ) {}

    static async new(): Promise<TestServer> {
        return new TestServer(
            env("AGDB_SERVER_DATA_DIR"),
            api_for_test(env("AGDB_SERVER_ADDRESS")),
        );
    }
}

/** The leader is the first node. */
export class TestCluster {
    constructor(
        public addresses: string[],
        public data_dirs: string[],
    ) {}

    static async new(): Promise<TestCluster> {
        return new TestCluster(
            env("AGDB_CLUSTER_ADDRESSES").split(","),
            env("AGDB_CLUSTER_DATA_DIRS").split(","),
        );
    }

    leader(): AgdbApi<FetchClient> {
        return api_for_test(this.addresses[0]);
    }

    follower(): AgdbApi<FetchClient> {
        return api_for_test(this.addresses[1]);
    }
}
