<script>
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import AuthorizedAppItem from "$comps/items/authorizedAppItem.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
</script>

<SettingsPanel
    title="Authorized Apps"
    description="The pieces of magic spaghetti that have access to your account. Revoke any app you don't recognise"
>
    <LogItem.Container>
        <AuthorizedAppItem />
        <AuthorizedAppItem />
        <AuthorizedAppItem />
        <AuthorizedAppItem />
        <AuthorizedAppItem />
        <AuthorizedAppItem />
        <AuthorizedAppItem />
        <AuthorizedAppItem />
    </LogItem.Container>
</SettingsPanel>
