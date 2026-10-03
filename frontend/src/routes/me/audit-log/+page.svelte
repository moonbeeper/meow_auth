<script lang="ts">
    import { goto } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import KeyTag from "$comps/keyTag.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import { currentUserAuditLog, getCurrentUserAuditLogQueryKey } from "$lib/api/user/user";
    import { actionCategory, actionLabel, intoAuditAction } from "$lib/auditLog";
    import { actionDate } from "$lib/common";
    import { KeysThatMatter } from "$lib/stupidKeymap";
    import SvelteVirtualList, {
        type SvelteVirtualListRangeInfo
    } from "@humanspeak/svelte-virtual-list";
    import { createInfiniteQuery } from "@tanstack/svelte-query";

    const auditLogQuery = createInfiniteQuery(() => ({
        queryKey: [...getCurrentUserAuditLogQueryKey(), "infinite"],
        queryFn: ({ pageParam, signal }) =>
            currentUserAuditLog({ from: pageParam, want_total: true }, { signal }),
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

    function onKeyDown() {
        goto("/");
    }

    let panelDescription = $derived.by(() => {
        const d = "A little trail of your recent activity on your account";
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

<div class="actions">
    <Button href="/me">Go back <KeyTag key={KeysThatMatter.Escape} {onKeyDown} /></Button>
</div>

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
    .actions {
        display: flex;
        inline-size: 100%;
    }

    .content {
        --vlist-viewport-height: 80dvh;
        // overflow: hidden;
        // height: 80dvh;
    }
</style>
