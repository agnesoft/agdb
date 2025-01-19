import { expect, describe, it } from "vitest";
import { render, screen } from "@testing-library/react";
import Intro from "@/components/pages/intro";

describe("intro", () => {
    it("should render the intro", () => {
        render(<Intro />);
        expect(screen.getByText(/First application native database without compromises/i)).toBeDefined();
    });
});
