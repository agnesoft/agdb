import { describe, it, expect, afterEach } from "vitest";
import axiosService from "@/services/axios.service";
import MockAdapter from "axios-mock-adapter";

const mock = new MockAdapter(axiosService);

describe("axiosService", () => {
    afterEach(() => {
        mock.reset();
    });

    it("should set the base URL and content type headers", () => {
        expect(axiosService.defaults.baseURL).toBe(process.env.VUE_APP_API_URL);
        expect(axiosService.defaults.headers.post["Content-Type"]).toBe("application/json");
    });

    it("should handle successful responses", async () => {
        const response = { data: "Success" };
        mock.onPost("/login").reply(200, response);
        const result = await axiosService.post("/login", { username: "admin", password: "admin" });
        expect(result).toEqual(response);
    });

    it("should handle network errors", async () => {
        mock.onPost("/login").networkErrorOnce();
        await expect(
            axiosService.post("/login", { username: "admin", password: "admin" }),
        ).rejects.toThrow("Network Error");
    });

    it("should handle 401 status code", async () => {
        mock.onPost("/login").reply(401);
        await expect(
            axiosService.post("/login", { username: "admin", password: "admin" }),
        ).rejects.toThrow("Request failed with status code 401");
    });

    it("should handle 302 status code", async () => {
        mock.onPost("/login").reply(302);
        await expect(
            axiosService.post("/login", { username: "admin", password: "admin" }),
        ).rejects.toThrow("Request failed with status code 302");
    });

    it("should handle 404 status code", async () => {
        mock.onPost("/login").reply(404);
        await expect(
            axiosService.post("/login", { username: "admin", password: "admin" }),
        ).rejects.toThrow("Request failed with status code 404");
    });

    it("should handle 500 status code", async () => {
        mock.onPost("/login").reply(500);
        await expect(
            axiosService.post("/login", { username: "admin", password: "admin" }),
        ).rejects.toThrow("Request failed with status code 500");
    });

    it("should handle error responses", async () => {
        const response = { message: "Error" };
        mock.onPost("/login").reply(400, response);
        await expect(
            axiosService.post("/login", { username: "admin", password: "admin" }),
        ).rejects.toThrow("Error");
    });

    it("should handle error responses with no message", async () => {
        mock.onPost("/login").reply(400);
        await expect(
            axiosService.post("/login", { username: "admin", password: "admin" }),
        ).rejects.toThrow("Unknown error");
    });
});
