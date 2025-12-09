import { useCallback, useState } from "react";
import CodeBlock from "@/components/common/code-block";

export const OpenApi = () => {
    const [openapiFile, setOpenapiFile] = useState<string>();
    const handleLoadCode = useCallback(() => {
        /* v8 ignore next -- @preserve */
        if (!openapiFile) {
            import("../../../../agdb_server/openapi.json").then((data) => {
                const openapiString = JSON.stringify(data.default, null, 2);
                setOpenapiFile(openapiString);
            });
        }
    }, [openapiFile]);
    return (
        <CodeBlock
            code={openapiFile}
            language="json"
            header="openapi.json"
            onLoad={handleLoadCode}
        />
    );
};
