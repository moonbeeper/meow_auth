import { API_URL } from "$app/env/public";
import createClient from "openapi-fetch";

import type { paths } from "./v1";

const client = createClient<paths>({
    baseUrl: API_URL,
    headers: {
        "Content-Type": "application/json"
    },
    credentials: "include"
});

export default client;
