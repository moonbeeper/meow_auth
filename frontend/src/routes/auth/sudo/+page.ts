import { isOk } from "$lib/api/ignoreThisPlease";
import { sudoEnableOptions, sudoOtpStart } from "$lib/api/sudo/sudo";
import { tryCatch } from "$lib/common";
import { redirect } from "@sveltejs/kit";

import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ url, fetch }) => {
    const redirectUrl = url.searchParams.get("redirect");

    // // oh that's actually nice, i didnt know you could just NOT import the types lol
    // let options: Awaited<ReturnType<typeof getEnableOptions>>;
    // try {
    //     options = await getEnableOptions(undefined, fetch);
    // } catch {
    //     console.error("couldnt begin talking to backend about sudo stuff (you know heh)");
    //     redirect(303, redirectUrl ?? "/");
    // }
    //
    const options = await tryCatch(() => sudoEnableOptions(undefined, fetch));
    if (options.error) {
        console.error("couldnt begin talking to backend about sudo stuff (you know heh)");
        redirect(303, redirectUrl ?? "/");
    }

    if (!isOk(options.result)) {
        if (options.result.data.code == "SudoAlreadyEnabled") {
            console.warn("sudo was already enabled");
            redirect(303, redirectUrl ?? "/");
        }
        console.error("could not get sudo enable options");
        redirect(303, redirectUrl ?? "/");
    }

    if (options.result.data.options.length == 1) {
        if (options.result.data.options.includes("otp")) {
            const otpData = await tryCatch(() => sudoOtpStart(undefined, fetch));
            if (otpData.error) {
                console.error("couldnt begin talking to backend about sudo stuff (you know heh)");
                redirect(303, redirectUrl ?? "/");
            }
            // let otpData: Awaited<ReturnType<typeof otpOption>>;
            // try {
            //     otpData = await otpOption(undefined, fetch);
            // } catch {
            //     console.error("couldnt begin talking to backend about sudo stuff (you know heh)");
            //     redirect(303, redirectUrl ?? "/");
            // }

            if (!isOk(otpData.result)) {
                if (otpData.result.data.code == "SudoAlreadyEnabled") {
                    console.warn("sudo was already enabled");
                    redirect(303, redirectUrl ?? "/");
                } else if (otpData.result.data.code == "SudoOptionNotAvailable") {
                    console.warn("sudo option not available");
                    redirect(303, redirectUrl ?? "/");
                }

                console.error("could not create otp flow");
                redirect(303, redirectUrl ?? "/");
            }
            const flowId = otpData.result.data.flow_id;
            let redirectProp = "";

            if (redirectUrl) {
                redirectProp = `&redirect=${encodeURIComponent(redirectUrl)}`;
            }

            redirect(303, `/auth/${flowId}/otp?sudo=true${redirectProp}`);
        }

        console.error("got only one option but it wasnt otp");
        redirect(303, redirectUrl ?? "/");
    }

    return {
        redirect: redirectUrl || null,
        options: options.result.data.options
    };
};
