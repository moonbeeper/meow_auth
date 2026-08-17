<script lang="ts">
    import EmptyBox from "$comps/emptyBox.svelte";
    import AuthorizedAppItem from "$comps/items/authorizedAppItem.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import {
        getListOauthAuthorizationsQueryKey,
        listOauthAuthorizations
    } from "$lib/api/oauth/oauth";
    import type { SvelteVirtualListRangeInfo } from "@humanspeak/svelte-virtual-list";
    import SvelteVirtualList from "@humanspeak/svelte-virtual-list";
    import { createInfiniteQuery } from "@tanstack/svelte-query";

    const authorizedAppsQuery = createInfiniteQuery(() => ({
        queryKey: [...getListOauthAuthorizationsQueryKey(), "infinite"],
        queryFn: ({ pageParam, signal }) =>
            listOauthAuthorizations({ from: pageParam, want_total: true }, { signal }),
        initialPageParam: undefined as string | undefined,
        getNextPageParam: (lastPage) => {
            if (lastPage.status !== 200) return undefined;
            return lastPage.data.next ?? undefined;
        }
    }));

    const authorizedAppsList = $derived.by(() => {
        const pages = authorizedAppsQuery.data?.pages ?? [];
        return pages.filter((p) => p.status === 200).flatMap((p) => p.data.data);
    });

    let panelDescription = $derived.by(() => {
        const d =
            "The pieces of magic spaghetti that have access to your account. Revoke any app you don't recognise";
        const pages = authorizedAppsQuery.data?.pages ?? [];
        const total = pages.filter((p) => p.status === 200)[0]?.data.total ?? -1;

        return d + (total > 0 ? ` (${total} total)` : "");
    });

    function handleRangeChange({ atBottom }: SvelteVirtualListRangeInfo) {
        if (
            atBottom &&
            authorizedAppsQuery.hasNextPage &&
            !authorizedAppsQuery.isFetchingNextPage
        ) {
            authorizedAppsQuery.fetchNextPage();
        }
    }
</script>

<SettingsPanel title="Authorized Apps" description={panelDescription}>
    {#if authorizedAppsQuery.isLoading || authorizedAppsQuery.isError}
        <LoadingBox
            what="authorized apps"
            loaded={!authorizedAppsQuery.isLoading}
            hasError={authorizedAppsQuery.isError}
        />
    {:else if authorizedAppsQuery.isSuccess}
        {#if authorizedAppsList.length != 0}
            <LogItem.Container>
                <div class="content">
                    <SvelteVirtualList
                        items={authorizedAppsList}
                        onRangeChange={handleRangeChange}
                        containerClass="vlist-container"
                        viewportClass="vlist-viewport"
                    >
                        {#snippet renderItem(app)}
                            <AuthorizedAppItem />
                        {/snippet}
                    </SvelteVirtualList>
                </div>
            </LogItem.Container>
        {:else}
            <EmptyBox
                title="No authorized apps"
                subtitle="No apps have access to your account yet. Once you authorize one, it'll appear right here so you can manage it later!"
            />
        {/if}
    {/if}
</SettingsPanel>

<style lang="scss">
    .content {
        --vlist-viewport-height: 80dvh;
    }
</style>
