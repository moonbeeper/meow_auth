import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async ({ url }) => {
    return {
        redirect: url.searchParams.get("redirect") || null
    };
};
