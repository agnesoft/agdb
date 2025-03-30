import { defineAsyncComponent } from "vue";

const asyncComponents: Record<
  AsyncComponent,
  ReturnType<typeof defineAsyncComponent>
> = {
  DbDetails: defineAsyncComponent(
    () => import("@/components/db/DbDetails.vue"),
  ),
};
export const getAsyncComponent = (componentName: AsyncComponent) => {
  return asyncComponents[componentName];
};
