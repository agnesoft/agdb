import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { LinkItem } from "./link-item";

describe("LinkItem", () => {
    it("should render the link item with correct link and text", () => {
        render(<LinkItem i18nKey="about" />);
        const link = screen.getByText("About");
        expect(link.getAttribute("href")).toBe("/about");
    });
});
