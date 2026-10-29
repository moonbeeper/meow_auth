<script lang="ts">
    import { invalidate } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";
    import DialogClose from "$comps/dialog/dialogClose.svelte";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import {
        createRotateSecretOauthApplication,
        getListOauthApplicationsQueryKey
    } from "#lib/api/index.js";
    import type { OauthApplication } from "#lib/api/model/index.js";
    import queryClient from "#lib/api/tanstackClient.js";

    export type Props = {
        item: OauthApplication;
        open: boolean;
    };

    let { item, open = $bindable(false) }: Props = $props();
    let currentStep = $state(0);

    let rotateSecretsApp = createRotateSecretOauthApplication(() => ({
        mutation: {
            onSuccess: () => {
                queryClient.invalidateQueries({ queryKey: getListOauthApplicationsQueryKey() });
            }
        }
    }));

    let display_name = $derived.by(() => {
        return item?.name ?? "Unknown";
    });

    function resetDialog() {
        open = false;
        app_secret = null;
        currentStep = 0;
    }

    let app_secret = $state<null | string>(null);
    async function doRotate() {
        if (!item) {
            return;
        }
        const res = await rotateSecretsApp.mutateAsync({
            id: item.id
        });

        if (!isOk(res)) {
            console.error("failed to rotate secrets for app " + item.id);
            resetDialog();
            return;
        }
        console.info("deleted app " + item.id);

        app_secret = res.data.secret;
        currentStep = 1;
        invalidate((url) => url.pathname === "/me/my-apps");
    }

    let title = $derived.by(() => {
        if (currentStep == 0) {
            return "Are you sure you want to rotate the secrets?";
        } else if (currentStep == 1) {
            return "New app secret generated";
        }
        return "Are you sure you want to delete this app?";
    });

    $effect(() => {
        if (!open) {
            resetDialog();
            return;
        }
    });
</script>

<Dialog.Root {title} bind:open>
    {#if currentStep == 0}
        <p>
            <b>{display_name}</b> will have its secrets rotated!! Clients using the old client secret
            will no longer work
        </p>
    {/if}

    {#if currentStep == 1}
        {#if app_secret}
            <p>Save your app secret now!! It won't be shown again.</p>
            <span class="secret">
                {app_secret}
            </span>
        {:else}
            <p>Failed to rotate the secret for <b>{display_name}</b>.</p>
        {/if}
    {/if}
    {#snippet actions()}
        {#if currentStep == 0}
            <Button
                negative
                shouldFill
                onclick={() => doRotate()}
                loading={rotateSecretsApp.isPending}
                disabled={rotateSecretsApp.isPending}>Yes, rotate them</Button
            >
        {/if}
        {#if currentStep == 1}
            <DialogClose shouldFill>Close</DialogClose>
        {/if}
    {/snippet}
</Dialog.Root>

<style lang="scss">
    .secret {
        font-size: var(--text-small);
        font-family: var(--font-mono);
        padding: 0.5rem;
        font-weight: 600;
        overflow-wrap: anywhere;
    }
</style>
