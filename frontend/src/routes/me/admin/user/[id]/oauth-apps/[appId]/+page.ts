import { error } from "@sveltejs/kit";
import z from "zod";

import { isOk } from "#lib/api/ignoreThisPlease.js";
import { getInfoOauthApplication } from "#lib/api/index.js";
import { tryCatch } from "#lib/common.js";

import type { PageLoad } from "./$types";

const schema = z.object({
    appId: z.ulid()
});

export const load: PageLoad = async ({ params, fetch }) => {
    const parsed = schema.safeParse({
        appId: params.appId
    });

    if (!parsed.success) {
        error(400, "unknown flow id");
    }

    const options = await tryCatch(() =>
        getInfoOauthApplication(parsed.data.appId, undefined, fetch)
    );
    if (options.error) {
        console.error("couldnt begin talking to backend about sudo stuff (you know heh)");
        throw new Error("oh god");
    }

    if (!isOk(options.result)) {
        if (options.result.data.code == "SudoAlreadyEnabled") {
            console.warn("sudo was already enabled");
            throw new Error("oh god");
        }
        console.error("could not get sudo enable options");
        throw new Error("oh god");
    }
    // queryClient.setQueryData(
    //     getGetInfoOauthApplicationQueryKey(parsed.data.appId),
    //     options.result.data
    // );

    console.log("options", options);

    return {
        app: options.result.data
    };
};
