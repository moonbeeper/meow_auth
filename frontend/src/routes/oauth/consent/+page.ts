import { oauthConsentInfo } from "$lib/api";
import { isOk } from "$lib/api/ignoreThisPlease";
import { tryCatch } from "$lib/common";
import { error, redirect } from "@sveltejs/kit";

import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch }) => {
    const options = await tryCatch(() => oauthConsentInfo(undefined, fetch));
    if (options.error) {
        console.error("couldnt begin talking to backend about oauth consent info");
        redirect(303, "/");
    }

    if (!isOk(options.result)) {
        if (options.result.data.code == "FlowNotFound") {
            console.warn("oauth flow not found");
            error(404, "This authorization request has expired or is invalid");
        } else {
            console.error("could not get oauth consent info");
            redirect(303, "/");
        }
    }

    return {
        consent: options.result.data
    };
};
