import "./mocks/apiMock";
import { config } from "@vue/test-utils";
import { vOnClickOutside } from "@vueuse/components";

config.global.directives = {
  ...config.global.directives,
  OnClickOutside: vOnClickOutside,
};

document.body.innerHTML = `
<head>
    <div id="app"></div>
</head>
`;
