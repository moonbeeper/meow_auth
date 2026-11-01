<script lang="ts">
    import EmptyBox from "$comps/emptyBox.svelte";
    import AuthorizedAppItem from "$comps/items/authorizedAppItem.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import type { SvelteVirtualListRangeInfo } from "@humanspeak/svelte-virtual-list";
    import SvelteVirtualList from "@humanspeak/svelte-virtual-list";
    import { IconSearch } from "@tabler/icons-svelte-runes";
    import { createInfiniteQuery } from "@tanstack/svelte-query";

    import { adminListUsers, getAdminListUsersQueryKey } from "#lib/api/index.js";
    import type { ListDataResponseOauthAuthorizationDataItem } from "#lib/api/model/index.js";
    import {
        getListOauthAuthorizationsQueryKey,
        listOauthAuthorizations
    } from "#lib/api/oauth/oauth.js";
    import Input from "#lib/components/form/input.svelte";
    import AdminUserItem from "#lib/components/items/adminUserItem.svelte";
    import Symbol from "#lib/components/symbol.svelte";

    let search = $state("");
    let debouncedSearch = $state("");

    $effect(() => {
        const v = search.trim();
        const timeout = setTimeout(() => {
            debouncedSearch = v;
        }, 300);
        return () => clearTimeout(timeout);
    });

    const usersQuery = createInfiniteQuery(() => ({
        queryKey: [
            ...getAdminListUsersQueryKey({ search: debouncedSearch, pagination: {} }),
            "infinite"
        ],
        queryFn: ({ pageParam, signal }) =>
            adminListUsers(
                { pagination: { from: pageParam, want_total: true }, search: debouncedSearch },
                { signal }
            ),
        initialPageParam: undefined as string | undefined,
        getNextPageParam: (lastPage) => {
            if (lastPage.status !== 200) return undefined;
            return lastPage.data.next ?? undefined;
        }
    }));

    const userList = $derived.by(() => {
        const pages = usersQuery.data?.pages ?? [];
        return pages.filter((p) => p.status === 200).flatMap((p) => p.data.data);
    });

    let panelDescription = $derived.by(() => {
        const d = "The list of registered users";
        const pages = usersQuery.data?.pages ?? [];
        const total = pages.filter((p) => p.status === 200)[0]?.data.total ?? -1;

        return d + (total > 0 ? ` (${total} total)` : "");
    });

    function handleRangeChange({ atBottom }: SvelteVirtualListRangeInfo) {
        if (atBottom && usersQuery.hasNextPage && !usersQuery.isFetchingNextPage) {
            usersQuery.fetchNextPage();
        }
    }
</script>

<svelte:head>
    <title>Admin - Meow Auth</title>
</svelte:head>

<SettingsPanel title="Registered users" description={panelDescription}>
    <div class="actions">
        <Symbol symbol={IconSearch} size="medium" />
        <Input placeholder="Search by user or email" bind:value={search} />
    </div>

    {#if usersQuery.isLoading || usersQuery.isError}
        <LoadingBox
            what="authorized apps"
            loaded={!usersQuery.isLoading}
            hasError={usersQuery.isError}
        />
    {:else if usersQuery.isSuccess}
        {#if userList.length != 0}
            <LogItem.Container>
                <div class="content">
                    {#if userList.length > 8}
                        <SvelteVirtualList
                            items={userList}
                            onRangeChange={handleRangeChange}
                            containerClass="vlist-container"
                            viewportClass="vlist-viewport"
                        >
                            {#snippet renderItem(app)}
                                <AdminUserItem item={app} />
                            {/snippet}
                        </SvelteVirtualList>
                    {:else}
                        {#each userList as app}
                            <AdminUserItem item={app} />
                        {/each}
                    {/if}
                </div>
            </LogItem.Container>
        {:else}
            <EmptyBox
                title="No users found"
                subtitle="There are no registered users matching your search criteria or... there are no users at all"
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
        gap: calc(var(--spacing) * 4);
    }
</style>
