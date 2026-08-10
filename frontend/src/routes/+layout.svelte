<script lang="ts">
    import { version } from "$app/env";
    import Header from "$comps/header.svelte";
    import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";

    import "../styles/_global.scss";
    import "@fontsource-variable/inter/wght.css";

    let { children } = $props();
    let build_date = new Date(Number(version)).toTimeString();
    // remember: cannot use crypto stuff [how would have known that, i didnt :(]

    const queryClient = new QueryClient();
</script>

<QueryClientProvider client={queryClient}>
    <Header />
    {@render children()}
    <footer class="footer">
        <div class="footer--content">
            Running on {__COMMIT_HASH__} built at {build_date}
        </div>
    </footer>
</QueryClientProvider>
