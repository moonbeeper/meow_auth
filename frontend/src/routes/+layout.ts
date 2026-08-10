import { auth } from "$lib/auth/auth.svelte";
import { getRedirectUrl } from "$lib/redirect";
import { redirect } from "@sveltejs/kit";

export const ssr = false;

export const load = async ({ fetch, url }) => {
    await auth.update(fetch);

    const redirectUrl = getRedirectUrl(auth.user, url);
    if (redirectUrl) {
        redirect(303, redirectUrl);
    }
};
