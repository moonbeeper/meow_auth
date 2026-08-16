import { QueryClient } from "@tanstack/svelte-query";

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            staleTime: 1000 * 60 * 1 // if not set, all queries will be shown as "stale" WHEN THEY ARE JUST FETCHED BRUh
        }
    }
});

export default queryClient;
