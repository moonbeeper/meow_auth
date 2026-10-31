<script lang="ts">
    import { goto, refreshAll } from "$app/navigation";
    import EmptyBox from "$comps/emptyBox.svelte";
    import SessionItem from "$comps/items/sessionItem.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import type { SvelteVirtualListRangeInfo } from "@humanspeak/svelte-virtual-list";
    import SvelteVirtualList from "@humanspeak/svelte-virtual-list";
    import { IconTrashFilled } from "@tabler/icons-svelte-runes";
    import { createInfiniteQuery } from "@tanstack/svelte-query";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import {
        createRevokeAllSessions,
        createRevokeSession,
        getListSessionsQueryKey,
        listSessions
    } from "#lib/api/sessions/sessions.js";
    import queryClient from "#lib/api/tanstackClient.js";
    import Button from "#lib/components/button.svelte";
    import Symbol from "#lib/components/symbol.svelte";

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

    const requestRevokeSession = createRevokeSession(() => ({
        mutation: {
            onSuccess: () => {
                queryClient.invalidateQueries({ queryKey: getListSessionsQueryKey() });
            }
        }
    }));

    const requestRevokeAllSessions = createRevokeAllSessions(() => ({
        mutation: {
            onSuccess: () => {
                queryClient.invalidateQueries({ queryKey: getListSessionsQueryKey() });
                refreshAll();
            }
        }
    }));

    async function handleOnDelete(id: string, isAll: boolean = false) {
        const res = isAll
            ? await requestRevokeAllSessions.mutateAsync()
            : await requestRevokeSession.mutateAsync({ id: id });

        if (!isOk(res)) {
            if (res.data.code == "SudoNotEnabled") {
                console.error("sudo not enabled, cannot revoke session. redirecting");
                await goto(`/auth/sudo?redirect=${encodeURIComponent("/me/security")}`);
            }

            console.error("failed to revoke session");
            return;
        }
    }

    // TODO: add session metadata.
</script>

<SettingsPanel title="Active sessions" description={panelDescription}>
    <div class="actions">
        <Button
            negative
            onclick={() => handleOnDelete("", true)}
            disabled={requestRevokeAllSessions.isPending}
            loading={requestRevokeAllSessions.isPending}
            symbol_prefix={{ symbol: IconTrashFilled }}>Revoke all</Button
        >

        <!-- <Symbol symbol={IconTrashFilled} /> -->
    </div>

    {#if sessionQuery.isLoading || sessionQuery.isError}
        <LoadingBox
            what="sessions"
            loaded={!sessionQuery.isLoading}
            hasError={sessionQuery.isError}
        />
    {:else if sessionQuery.isSuccess}
        {#if sessionList.length != 0}
            <LogItem.Container collapsible>
                <div class="content">
                    {#if sessionList.length > 8}
                        <SvelteVirtualList
                            items={sessionList}
                            onRangeChange={handleRangeChange}
                            containerClass="vlist-container"
                            viewportClass="vlist-viewport"
                        >
                            {#snippet renderItem(session)}
                                <SessionItem item={session} onDelete={handleOnDelete} />
                            {/snippet}
                        </SvelteVirtualList>
                    {:else}
                        {#each sessionList as session}
                            <SessionItem item={session} onDelete={handleOnDelete} />
                        {/each}
                    {/if}
                </div>
            </LogItem.Container>
        {:else}
            <EmptyBox
                title="No open sessions yet"
                subtitle="That is actually a strange thing to see. You should have at least one session open, which is this one... are you a magic intruder?"
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
