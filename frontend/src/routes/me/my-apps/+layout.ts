import { redirect } from "@sveltejs/kit";

import { auth } from "#lib/auth/auth.svelte.js";
import { UserFlag } from "#lib/auth/userFlags.js";

import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async ({ parent }) => {
    await parent();
    const user = auth.user;
    if (!user) {
        redirect(303, "/");
    }
    if (user.flags.has(UserFlag.CannotManageOauthApplications)) {
        redirect(303, "/me");
    }
};
