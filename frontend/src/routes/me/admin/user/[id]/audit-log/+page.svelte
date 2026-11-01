<script lang="ts">
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import SvelteVirtualList, {
        type SvelteVirtualListRangeInfo
    } from "@humanspeak/svelte-virtual-list";
    import { createInfiniteQuery } from "@tanstack/svelte-query";

    import { adminUserAuditLog, getAdminUserAuditLogQueryKey } from "#lib/api/index.js";

    import type { LayoutProps } from "../$types";

    let { data }: LayoutProps = $props();
    let userInfo = $derived(data.user);

    const auditLogQuery = createInfiniteQuery(() => ({
        queryKey: [...getAdminUserAuditLogQueryKey(userInfo.id), "infinite"],
        queryFn: ({ pageParam, signal }) =>
            adminUserAuditLog(userInfo.id, { from: pageParam, want_total: true }, { signal }),
        initialPageParam: undefined as string | undefined,
        getNextPageParam: (lastPage) => {
            if (lastPage.status !== 200) return undefined;
            return lastPage.data.next ?? undefined;
        }
    }));

    const auditLogList = $derived.by(() => {
        const pages = auditLogQuery.data?.pages ?? [];
        return pages.filter((p) => p.status === 200).flatMap((p) => p.data.data);
    });

    let panelDescription = $derived.by(() => {
        const d = "The little trail of recent activity of this user on their account";
        const pages = auditLogQuery.data?.pages ?? [];
        const total = pages.filter((p) => p.status === 200)[0]?.data.total ?? -1;

        return d + (total > 0 ? ` (${total} total)` : "");
    });

    function handleRangeChange({ atBottom }: SvelteVirtualListRangeInfo) {
        if (atBottom && auditLogQuery.hasNextPage && !auditLogQuery.isFetchingNextPage) {
            auditLogQuery.fetchNextPage();
        }
    }
</script>

<svelte:head>
    <title>Audit Log - Meow Auth</title>
</svelte:head>

<!-- has to be inside a div to see that its the only child.. DUH DUMB BIRD I MA GODDAMIT -->
<div>
    <SettingsPanel title="Audit log" description={panelDescription}>
        {#if auditLogQuery.isLoading || auditLogQuery.isError}
            <LoadingBox loaded={!auditLogQuery.isLoading} hasError={auditLogQuery.isError} />
        {:else if auditLogQuery.isSuccess}
            <LogItem.Container collapsible>
                <div class="content">
                    <SvelteVirtualList
                        items={auditLogList}
                        onRangeChange={handleRangeChange}
                        containerClass="vlist-container"
                        viewportClass="vlist-viewport"
                    >
                        {#snippet renderItem(logItem)}
                            <AuditLogItem2 item={logItem} />
                        {/snippet}
                    </SvelteVirtualList>
                </div>
            </LogItem.Container>
        {/if}
    </SettingsPanel>
</div>

<style lang="scss">
    .content {
        --vlist-viewport-height: 80dvh;
        // overflow: hidden;
        // height: 80dvh;
    }
</style>
