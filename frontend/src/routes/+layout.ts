import { auth } from "$lib/auth.svelte";

export const ssr = false;

export const load = async ({ fetch }) => {
    await auth.update(fetch);
};
