import React from "react";
import openapiFile from "../../../../agdb_server/openapi.json";
import CodeBlock from "@/components/common/code-block";

export const OpenApi = () => {
    const openapiJson = JSON.stringify(openapiFile, null, 2);
    return (
        <CodeBlock code={openapiJson} language="json" header="openapi.json" />
    );
};
