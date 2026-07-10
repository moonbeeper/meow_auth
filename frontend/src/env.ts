import { defineEnvVars } from "@sveltejs/kit/hooks";

export const variables = defineEnvVars({
    API_URL: {
        public: true
    }
});
