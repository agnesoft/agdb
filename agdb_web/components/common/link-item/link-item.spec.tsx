import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { LinkItem } from "./link-item";

// vi.mock("next/router", () => ({
//     useRouter: () => ({
//         pathname: "/",
//         locale: "en-US",
//     }),
// }));

// vi.hoisted(() => {
vi.mock("@/hooks/i18n", () => ({
    useI18n: () => ({
        t: (key: string) => {
            if (key === "url.about") return "/about";
            if (key === "link.about") return "About";
            return "";
        },
    }),
}));
// });

describe("LinkItem", () => {
    it("should render the link item with correct link and text", () => {
        render(<LinkItem i18nKey="about" />);
        const link = screen.getByText("About");
        expect(link.getAttribute("href")).toBe("/about");
    });
});
