<script lang="ts">
    import EmptyBox from "$comps/emptyBox.svelte";
    import CreatedAppItem from "$comps/items/createdAppItem.svelte";
    import SessionItem from "$comps/items/sessionItem.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import {
        getListApplicationsQueryKey,
        getListOauthAuthorizationsQueryKey,
        listApplications
    } from "$lib/api/oauth/oauth";
    import { getListSessionsQueryKey, listSessions } from "$lib/api/sessions/sessions";
    import { actionDate } from "$lib/common";
    import type { SvelteVirtualListRangeInfo } from "@humanspeak/svelte-virtual-list";
    import SvelteVirtualList from "@humanspeak/svelte-virtual-list";
    import { createInfiniteQuery } from "@tanstack/svelte-query";

    const createdAppsQuery = createInfiniteQuery(() => ({
        queryKey: [...getListApplicationsQueryKey(), "infinite"],
        queryFn: ({ pageParam, signal }) =>
            listApplications({ from: pageParam, want_total: true }, { signal }),
        initialPageParam: undefined as string | undefined,
        getNextPageParam: (lastPage) => {
            if (lastPage.status !== 200) return undefined;
            return lastPage.data.next ?? undefined;
        }
    }));

    const createdAppsList = $derived.by(() => {
        const pages = createdAppsQuery.data?.pages ?? [];
        return pages.filter((p) => p.status === 200).flatMap((p) => p.data.data);
    });

    let panelDescription = $derived.by(() => {
        const d = "Apps you've brought into this world. They're your digital children!";
        const pages = createdAppsQuery.data?.pages ?? [];
        const total = pages.filter((p) => p.status === 200)[0]?.data.total ?? -1;

        return d + (total > 3 ? ` (${total} total)` : "");
    });

    function handleRangeChange({ atBottom }: SvelteVirtualListRangeInfo) {
        if (atBottom && createdAppsQuery.hasNextPage && !createdAppsQuery.isFetchingNextPage) {
            createdAppsQuery.fetchNextPage();
        }
    }
    // TODO: add session metadata.
</script>

<SettingsPanel title="Created Apps" description={panelDescription} contentSpacing={2}>
    {#if createdAppsQuery.isLoading || createdAppsQuery.isError}
        <LoadingBox
            what="created apps"
            loaded={!createdAppsQuery.isLoading}
            hasError={createdAppsQuery.isError}
        />
    {:else if createdAppsQuery.isSuccess}
        {#if createdAppsList.length != 0}
            <LogItem.Container>
                <div class="content">
                    <SvelteVirtualList
                        items={createdAppsList}
                        onRangeChange={handleRangeChange}
                        containerClass="vlist-container"
                        viewportClass="vlist-viewport"
                    >
                        {#snippet renderItem(app)}
                            <CreatedAppItem />
                        {/snippet}
                    </SvelteVirtualList>
                </div>
            </LogItem.Container>
        {:else}
            <EmptyBox
                title="No little apps yet!"
                subtitle="You haven't brought any apps into this digital world yet. Create one and it'll show up right here, ready to be looked after"
            />
        {/if}
    {/if}
</SettingsPanel>

<style lang="scss">
    .content {
        --vlist-viewport-height: 80dvh;
    }
</style>
