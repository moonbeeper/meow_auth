import { defineConfig } from "orval";

export default defineConfig({
    mrawpi: {
        output: {
            mode: "tags-split",
            target: "src/lib/api/mrawpi.ts",
            schemas: "src/lib/api/model",
            client: "svelte-query",
            baseUrl: {
                runtime: "apiUrlForOrval",
                imports: [{ name: "apiUrlForOrval", importPath: "$lib/api/ignoreThisPlease" }]
            },
            override: {
                fetch: {
                    useRuntimeFetcher: true
                },
                requestOptions: {
                    credentials: "include"
                }
            }
        },
        input: {
            target: ["./meow_auth.yaml", "http://127.0.0.1:8080/api-docs/openapi.json"]
        },
        hooks: {
            afterAllFilesWrite: "oxfmt"
        }
    }
});
