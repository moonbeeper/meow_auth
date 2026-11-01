import { error } from "@sveltejs/kit";
import z from "zod";

import { isOk } from "#lib/api/ignoreThisPlease.js";
import { adminInfoUser } from "#lib/api/index.js";
import { tryCatch } from "#lib/common.js";

import type { LayoutLoad } from "./$types";

const schema = z.object({
    user_id: z.ulid()
});

export const load: LayoutLoad = async ({ params, fetch }) => {
    const parsed = schema.safeParse({
        user_id: params.id
    });

    if (!parsed.success) {
        error(400, "unknown flow id");
    }

    const options = await tryCatch(() => adminInfoUser(parsed.data.user_id, undefined, fetch));
    if (options.error) {
        console.error("couldnt begin talking to backend about sudo stuff (you know heh)");
        throw new Error("oh god");
    }

    if (!isOk(options.result)) {
        console.error("could not get sudo enable options");
        throw new Error("oh god");
    }
    // queryClient.setQueryData(
    //     getGetInfoOauthApplicationQueryKey(parsed.data.appId),
    //     options.result.data
    // );

    return {
        user: options.result.data
    };
};
