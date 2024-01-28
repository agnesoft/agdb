import { expect, describe, it } from "vitest";
import { render, screen } from "@testing-library/react";
import Logo from "@/components/layout/logo";

describe("logo", () => {
    it("should render the logo", () => {
        render(<Logo />);
        expect(screen.getByText("agdb")).toBeDefined();
    });
});
