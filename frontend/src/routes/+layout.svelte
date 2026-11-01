<script lang="ts">
    import { version } from "$app/env";
    import Header from "$comps/header/header.svelte";
    import { QueryClientProvider } from "@tanstack/svelte-query";
    import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools";

    import "../styles/_global.scss";
    import "@fontsource-variable/inter/wght.css";

    import queryClient from "#lib/api/tanstackClient.js";

    let { children } = $props();
    let build_date = new Date(Number(version)).toTimeString();
    // remember: cannot use crypto stuff [how would have known that, i didnt :(]
    // const userQuery = createCurrentUserInfo();
    // $effect(() => {
    //     auth.sync(userQuery.data, userQuery.isPending);
    // });
</script>

<svelte:head>
    <title>Meow Auth</title>
</svelte:head>

<QueryClientProvider client={queryClient}>
    <Header />
    {@render children()}
    <footer class="footer">
        <div class="footer--content">
            Running on {__COMMIT_HASH__} built at {build_date}
        </div>
    </footer>
    <SvelteQueryDevtools />
</QueryClientProvider>
