import client from "./api/client";
import type { components } from "./api/v1";

/// its the auth context from the backend, but in the frontend!
class AuthState {
    user = $state<components["schemas"]["User"] | null>(null);
    loading = $state(true);

    async update(fetcher: typeof globalThis.fetch) {
        this.loading = true;
        try {
            const res = await client.GET("/v1/me", { fetch: fetcher });

            if (res.response.ok && res.data) {
                this.user = res.data;
                console.log("user has a session");
            } else {
                this.user = null;
                console.log("user doesn't have a session");
            }
        } catch (err) {
            console.error("something went wrong while updating the auth state: ", err);
            this.user = null;
        } finally {
            this.loading = false;
        }
    }
}

export const auth = new AuthState(); // ew, forced ordering.
