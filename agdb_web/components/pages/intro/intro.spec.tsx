import { expect, describe, it } from "vitest";
import { render, screen } from "@testing-library/react";
import Intro from "@/components/pages/intro";

describe("intro", () => {
    it("should render the intro", () => {
        render(<Intro />);
        expect(
            screen.getByText(/Application native database for any use case./i),
        ).toBeDefined();
    });
});
