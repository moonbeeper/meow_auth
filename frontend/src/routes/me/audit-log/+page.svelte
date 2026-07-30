<script lang="ts">
    import Button from "$comps/button.svelte";
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
</script>

<!-- TODO: id like to have esc behaviur here lol -->
<div class="actions">
    <Button href="/me">Go back</Button>
</div>
<!-- has to be inside a div to see that its the only child.. DUH DUMB BIRD I MA GODDAMIT -->
<div>
    <SettingsPanel
        title="Audit log"
        description="A little trail of your recent activity on your account"
    >
        <LogItem.Container>
            <AuditLogItem2 title="Created session" tag="Auth" who="you" when="2h ago">
                <span>hi this is metadata or just uselss</span>
            </AuditLogItem2>
            <AuditLogItem2 />
            <AuditLogItem2 />
            <AuditLogItem2 />
            <AuditLogItem2 />
            <AuditLogItem2 />
            <AuditLogItem2 />
            <AuditLogItem2 />
        </LogItem.Container>
    </SettingsPanel>
</div>

<style lang="scss">
    .actions {
        display: flex;
        inline-size: 100%;
    }
</style>
