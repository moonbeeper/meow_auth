import { redirect } from "@sveltejs/kit";

import { auth } from "#lib/auth/auth.svelte.js";
import { getRedirectUrl } from "#lib/redirect.js";

export const ssr = false;

export const load = async ({ fetch, url }) => {
    await auth.fetchAndSynctan(fetch);

    const redirectUrl = getRedirectUrl(auth.user, url);
    if (redirectUrl) {
        redirect(303, redirectUrl);
    }
};
