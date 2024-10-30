import { ref, computed } from "vue";
import { defineStore } from "pinia";

// this is just example store and will be removed

export const useCounterStore = defineStore("counter", () => {
    const count = ref(0);
    const doubleCount = computed(() => count.value * 2);
    function increment() {
        count.value++;
    }

    return { count, doubleCount, increment };
});
