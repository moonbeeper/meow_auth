<script>
    import Button from "$comps/button.svelte";
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import PasskeyItem from "$comps/items/passkeyItem.svelte";
    import SessionItem from "$comps/items/sessionItem.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
</script>

<SettingsPanel
    title="Two-Factor Authentication"
    description="Add a second layer of protection to your account! When enabled, you'll need a code from
your authentictor app to sign in"
>
    <div class="flex align-center">
        <Button primary>Disable</Button>
        <Button primary>View recovery codes</Button>
    </div>
</SettingsPanel>

<SettingsPanel title="Passkeys" description="Ah, efficient sign in. Modern and more secure!">
    <LogItem.Container>
        <PasskeyItem />
        <PasskeyItem />
        <PasskeyItem />
        <PasskeyItem />
        <PasskeyItem />
        <PasskeyItem />
        <PasskeyItem />
        <PasskeyItem />
    </LogItem.Container>
</SettingsPanel>

<SettingsPanel
    title="Active sessions"
    description="These devices are currently signed in to your account. Revoke any session you
            don't recognise"
>
    <LogItem.Container>
        <SessionItem />
        <SessionItem />
        <SessionItem />
        <SessionItem />
        <SessionItem />
        <SessionItem />
        <SessionItem />
        <SessionItem />
    </LogItem.Container>
</SettingsPanel>

<style lang="scss">
    .aa {
        position: relative;
        width: 96px;
        height: 96px;
        aspect-ratio: 1;
        /* border: 1px solid var(--color-iron-dark); */

        &::before {
            position: absolute;
            content: "";
            width: 100%;
            height: 100%;
            border-radius: 50%;
            border: 1px solid var(--color-iron-darkest);
        }
    }

    @media (max-width: 768px) {
        .baba {
            flex-direction: column;
            align-items: center;
        }
    }
</style>
