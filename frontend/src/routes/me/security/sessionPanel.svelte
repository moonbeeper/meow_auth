<script lang="ts">
    import EmptyBox from "$comps/emptyBox.svelte";
    import SessionItem from "$comps/items/sessionItem.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import { getListSessionsQueryKey, listSessions } from "$lib/api/sessions/sessions";
    import { actionDate } from "$lib/common";
    import type { SvelteVirtualListRangeInfo } from "@humanspeak/svelte-virtual-list";
    import SvelteVirtualList from "@humanspeak/svelte-virtual-list";
    import { createInfiniteQuery } from "@tanstack/svelte-query";

    const sessionQuery = createInfiniteQuery(() => ({
        queryKey: [...getListSessionsQueryKey(), "infinite"],
        queryFn: ({ pageParam, signal }) =>
            listSessions({ from: pageParam, want_total: true }, { signal }),
        initialPageParam: undefined as string | undefined,
        getNextPageParam: (lastPage) => {
            if (lastPage.status !== 200) return undefined;
            return lastPage.data.next ?? undefined;
        }
    }));

    const sessionList = $derived.by(() => {
        const pages = sessionQuery.data?.pages ?? [];
        return pages.filter((p) => p.status === 200).flatMap((p) => p.data.data);
    });

    let panelDescription = $derived.by(() => {
        const d =
            "These devices are currently signed in to your account. Revoke any session you don't recognise";
        const pages = sessionQuery.data?.pages ?? [];
        const total = pages.filter((p) => p.status === 200)[0]?.data.total ?? -1;

        return d + (total > 3 ? ` (${total} total)` : "");
    });

    function handleRangeChange({ atBottom }: SvelteVirtualListRangeInfo) {
        if (atBottom && sessionQuery.hasNextPage && !sessionQuery.isFetchingNextPage) {
            sessionQuery.fetchNextPage();
        }
    }
    // TODO: add session metadata.
</script>

<SettingsPanel title="Active sessions" description={panelDescription}>
    {#if sessionQuery.isLoading || sessionQuery.isError}
        <LoadingBox
            what="sessions"
            loaded={!sessionQuery.isLoading}
            hasError={sessionQuery.isError}
        />
    {:else if sessionQuery.isSuccess}
        {#if sessionList.length != 0}
            <LogItem.Container>
                <SvelteVirtualList
                    items={sessionList}
                    onRangeChange={handleRangeChange}
                    containerClass="vlist-container"
                    viewportClass="vlist-viewport"
                >
                    {#snippet renderItem(session)}
                        <SessionItem when={actionDate(session.created_at)} />
                    {/snippet}
                </SvelteVirtualList>
            </LogItem.Container>
        {:else}
            <EmptyBox
                title="No open sessions yet"
                subtitle="That is actually a strange thing to see. You should have at least one session open, which is this one... are you a magic intruder?"
            />
        {/if}
    {/if}
</SettingsPanel>
