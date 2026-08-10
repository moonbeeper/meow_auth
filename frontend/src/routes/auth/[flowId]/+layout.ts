import { auth } from "$lib/auth/auth.svelte";
import { error, redirect } from "@sveltejs/kit";
import z from "zod";

import type { LayoutLoad } from "./$types";

const schema = z.object({
    flow_id: z.ulid(),
    sudo: z.boolean().optional()
});

export const load: LayoutLoad = async ({ params, url, fetch }) => {
    await auth.update(fetch);
    const parsed = schema.safeParse({
        flow_id: params.flowId,
        sudo: url.searchParams.get("sudo") ?? false
    });

    if (!parsed.success) {
        error(400, "unknown flow id");
    }

    if (!parsed.data.sudo && auth.user) {
        console.warn("user is already logged in, redirecting to /me");
        redirect(303, "/");
    }

    return {
        flowId: parsed.data.flow_id,
        sudo: parsed.data.sudo
    };
};
