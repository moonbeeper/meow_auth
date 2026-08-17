<script>
    import Button from "$comps/button.svelte";
    import SettingsPanel from "$comps/settingsPanel.svelte";

    import PasskeyPanel from "./passkeyPanel.svelte";
    import SessionPanel from "./sessionPanel.svelte";
    import TotpPanel from "./totpPanel.svelte";
</script>

<TotpPanel />

<PasskeyPanel />

<SessionPanel />
