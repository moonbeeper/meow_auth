<script lang="ts">
    import Button from "$comps/button.svelte";
    import EmptyBox from "$comps/emptyBox.svelte";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import Spinner from "$comps/spinner.svelte";
    import { auth } from "$lib/auth/auth.svelte";

    let isTotpEnabled = $derived.by(() => {
        return auth.user?.has_totp ?? false;
    });
</script>

<!-- square buttons may look better. they eat more space -->
<SettingsPanel
    title="Two-Factor Authentication"
    description="Add a second layer of protection to your account! When enabled, you'll need a code from
your authentictor app to sign in"
>
    {#if !isTotpEnabled}
        <EmptyBox
            title="A little extra security?"
            subtitle="You don't have 2FA set up yet! Add it in just a moment and give your account a little extra protection"
        >
            {#snippet actions()}
                <Button primary fontSize="small">Set up 2FA</Button>
            {/snippet}
        </EmptyBox>
    {:else}
        <div class="actions">
            <Button square primary><Spinner /> Disable</Button>
            <Button square><Spinner /> View recovery codes</Button>
        </div>
    {/if}
</SettingsPanel>

<style lang="scss">
    .actions {
        display: flex;
        gap: calc(var(--spacing) * 2);
        align-items: center;
        max-inline-size: 100%;
        flex-wrap: wrap;

        @media (max-width: 376px) {
            flex-direction: column;
            align-items: stretch;
        }
    }
</style>
