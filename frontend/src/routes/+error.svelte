<script lang="ts">
import { page } from "$app/state";

const errors: Record<number, string> = {
	500: "Oh god. Something made the server implode :(",
	404: "Lost? same. This page doesn't exist!1!",
};

let message = $derived(page.error?.message ?? "An unexpected error occurred");
let status = $derived(page.status ?? 500);
let header = $derived(errors[status] ?? "Something weird happened1!!");
</script>

<svelte:head>
    <title>Oops! | MeowAuth</title>
</svelte:head>

<main id="main" class="grid justify-center margin-auto">
    <div class="flex flex-col align-center gap-3">
        <h1 class="font-xx-large">
            {header}
        </h1>

        <p class="font-medium">
            {status} - {message}
        </p>
    </div>
</main>
