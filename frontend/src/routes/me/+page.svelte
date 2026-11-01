<script lang="ts">
    import Button from "$comps/button.svelte";
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import { IconCaretRightFilled, IconTrashXFilled } from "@tabler/icons-svelte-runes";

    import { createCurrentUserAuditLog } from "#lib/api/user/user.js";

    import ProfilePanel from "./profilePanel.svelte";

    const auditLogQuery = createCurrentUserAuditLog();

    const recentAuditLog = $derived.by(() => {
        if (auditLogQuery.data?.status != 200) return [];
        const data = auditLogQuery.data.data.data ?? [];

        return data.slice(0, 8);
    });

    let isAuditLogLong = $derived.by(() => {
        return recentAuditLog.length >= 8;
    });
</script>

<svelte:head>
    <title>Me - Meow Auth</title>
</svelte:head>

<ProfilePanel />

<SettingsPanel
    title="Audit log"
    description="A little trail of your recent activity on your account"
>
    {#if auditLogQuery.isLoading || auditLogQuery.isError}
        <LoadingBox loaded={!auditLogQuery.isLoading} hasError={auditLogQuery.isError} />
    {:else if auditLogQuery.isSuccess}
        <LogItem.Container collapsible>
            {#each recentAuditLog as logItem (logItem.id)}
                <AuditLogItem2 item={logItem} />
            {/each}
        </LogItem.Container>
    {/if}
    {#if isAuditLogLong}
        <Button href="/me/audit-log" symbol_suffix={{ symbol: IconCaretRightFilled }}
            >View all</Button
        >
    {/if}
</SettingsPanel>

<SettingsPanel
    title="Delete account"
    description="Delete your account and everything associated with it, including authorized apps and created apps."
    negative
>
    <Button negative symbol_prefix={{ symbol: IconTrashXFilled }}>Delete account</Button>
</SettingsPanel>
