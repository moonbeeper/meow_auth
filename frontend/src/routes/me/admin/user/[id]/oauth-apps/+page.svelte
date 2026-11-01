<script lang="ts">
    import { goto } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import EmptyBox from "$comps/emptyBox.svelte";
    import CreatedAppItem from "$comps/items/createdAppItem.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import type { SvelteVirtualListRangeInfo } from "@humanspeak/svelte-virtual-list";
    import SvelteVirtualList from "@humanspeak/svelte-virtual-list";
    import { IconPlusFilled } from "@tabler/icons-svelte-runes";
    import { createInfiniteQuery } from "@tanstack/svelte-query";

    import {
        adminListUserApplications,
        getAdminListUserApplicationsQueryKey,
        getListOauthApplicationsQueryKey,
        listOauthApplications
    } from "#lib/api/index.js";
    import AdminCreatedAppItem from "#lib/components/items/adminCreatedAppItem.svelte";

    import type { LayoutProps } from "../$types";

    let { data }: LayoutProps = $props();
    let userInfo = $derived(data.user);

    const createdAppsQuery = createInfiniteQuery(() => ({
        queryKey: [...getAdminListUserApplicationsQueryKey(userInfo.id), "infinite"],
        queryFn: ({ pageParam, signal }) =>
            adminListUserApplications(
                userInfo.id,
                { from: pageParam, want_total: true },
                { signal }
            ),
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
        const d =
            "These are the apps that this user has brought into this world. They're their digital children!";
        const pages = createdAppsQuery.data?.pages ?? [];
        const total = pages.filter((p) => p.status === 200)[0]?.data.total ?? -1;

        return d + (total > 1 ? ` (${total} total)` : "");
    });

    function handleRangeChange({ atBottom }: SvelteVirtualListRangeInfo) {
        if (atBottom && createdAppsQuery.hasNextPage && !createdAppsQuery.isFetchingNextPage) {
            createdAppsQuery.fetchNextPage();
        }
    }
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
                    {#if createdAppsList.length > 8}
                        <SvelteVirtualList
                            items={createdAppsList}
                            onRangeChange={handleRangeChange}
                            containerClass="vlist-container"
                            viewportClass="vlist-viewport"
                        >
                            {#snippet renderItem(app)}
                                <AdminCreatedAppItem item={app} userId={userInfo.id} />
                            {/snippet}
                        </SvelteVirtualList>
                    {:else}
                        {#each createdAppsList as app}
                            <AdminCreatedAppItem item={app} userId={userInfo.id} />
                        {/each}
                    {/if}
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
    .actions {
        display: flex;
        inline-size: 100%;
        justify-content: end;
        align-items: center;
    }
</style>
