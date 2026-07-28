<script>
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import CreatedAppItem from "$comps/items/createdAppItem.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
</script>

<SettingsPanel
    title="Created Apps"
    description="Apps you've brought into this world. They're your digital children!"
>
    <LogItem.Container>
        <CreatedAppItem />
        <CreatedAppItem />
        <CreatedAppItem />
        <CreatedAppItem />
        <CreatedAppItem />
        <CreatedAppItem />
        <CreatedAppItem />
        <CreatedAppItem />
    </LogItem.Container>
</SettingsPanel>
