<script lang="ts">
    import { goto } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";
    import DialogClose from "$comps/dialog/dialogClose.svelte";

    import type { OauthApplicationDataResponse } from "#lib/api/model/index.js";

    export type Props = {
        item: OauthApplicationDataResponse | null;
        open: boolean;
    };

    let { item, open = $bindable(false) }: Props = $props();

    function resetDialog() {
        open = false;
    }

    function handleGoToApp() {
        if (!item) return;

        goto(`/me/my-apps/${item.id}`);
    }

    function handleGoBack() {
        goto("/me/my-apps/");
    }

    $effect(() => {
        if (!open) {
            resetDialog();
            return;
        }
    });
</script>

<Dialog.Root title="OAuth application created" bind:open preventClosing>
    <p>Save your app secret now!! It won't be shown again.</p>
    <div class="credentials">
        <div class="credential">
            <span class="credential--label">Application ID</span>
            <span class="secret">{item?.id ?? "Unknown"}</span>
        </div>
        <div class="credential">
            <span class="credential--label">Client secret</span>
            <span class="secret">{item?.secret ?? "Unknown"}</span>
        </div>
    </div>

    {#snippet actions()}
        <Button onclick={() => handleGoBack()}>Close</Button>
        <Button primary onclick={() => handleGoToApp()}>Go to app</Button>
    {/snippet}
</Dialog.Root>

<style lang="scss">
    .credentials {
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * 2);
    }

    .credential {
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * 1);
        font-size: var(--text-small);
    }

    .credential--label {
        color: var(--color-iron-dark);
    }

    .secret {
        font-family: var(--font-mono);
        padding: 0.5rem;
        font-weight: 600;
        overflow-wrap: anywhere;
    }
</style>
