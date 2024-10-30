import { useCounterStore } from "./counter";
import { setActivePinia, createPinia } from "pinia";

describe("counter", () => {
    beforeEach(() => {
        setActivePinia(createPinia());
    });
    it("increments the count", () => {
        const counter = useCounterStore();
        counter.increment();
        expect(counter.count).toBe(1);
    });
    it("doubles the count", () => {
        const counter = useCounterStore();
        counter.increment();
        expect(counter.doubleCount).toBe(2);
    });
});
