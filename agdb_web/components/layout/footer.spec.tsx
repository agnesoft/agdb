import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import Footer from "@/components/layout/footer";

vi.mock("@/hooks/i18n", () => ({
    useI18n: () => ({
        t: (key: string) => key,
    }),
}));

describe("footer", () => {
    it("should render the footer", () => {
        render(<Footer />);
        expect(screen.getByText(/copyright/i)).toBeDefined();
    });
});
