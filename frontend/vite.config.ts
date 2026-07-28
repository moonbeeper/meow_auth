import { execSync } from "node:child_process";

import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

const commit_hash = execSync("git rev-parse --short HEAD").toString().trim();
export default defineConfig({
    define: {
        __COMMIT_HASH__: JSON.stringify(commit_hash)
    },
    plugins: [
        sveltekit({
            preprocess: vitePreprocess(),
            compilerOptions: {
                // Force runes mode for the project, except for libraries. Can be removed in svelte 6.
                runes: ({ filename }) =>
                    filename.split(/[/\\]/).includes("node_modules") ? undefined : true
            },
            experimental: {
                explicitEnvironmentVariables: true
            },
            alias: {
                $comps: "src/lib/components"
            },
            // adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
            // If your environment is not supported, or you settled on a specific environment, switch out the adapter.
            // See https://svelte.dev/docs/kit/adapters for more information about adapters.
            adapter: adapter({})
        })
    ]
});
