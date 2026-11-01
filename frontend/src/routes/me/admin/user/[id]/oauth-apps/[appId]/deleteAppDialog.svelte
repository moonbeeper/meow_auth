<script lang="ts">
    import { goto } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import {
        createAdminDeleteUserApplication,
        createDeleteOauthApplication,
        getListOauthApplicationsQueryKey
    } from "#lib/api/index.js";
    import type { OauthApplication } from "#lib/api/model/index.js";
    import queryClient from "#lib/api/tanstackClient.js";

    export type Props = {
        item: OauthApplication;
        userId: string;
        open: boolean;
    };

    let { item, userId, open = $bindable(false) }: Props = $props();
    let currentStep = $state(0);

    let deleteApp = createAdminDeleteUserApplication();

    let display_name = $derived.by(() => {
        return item?.name ?? "Unknown";
    });

    function resetDialog() {
        open = false;
    }

    async function doRevoke() {
        if (!item) {
            return;
        }
        const res = await deleteApp.mutateAsync({
            id: userId,
            cid: item.id
        });

        if (!isOk(res)) {
            console.error("failed to delete app");
        } else {
            console.info("deleted app " + item.id);
        }

        resetDialog();
        await goto(`/me/admin/user/${userId}/oauth-apps`);
    }

    $effect(() => {
        if (!open) {
            resetDialog();
            return;
        }
    });
</script>

<Dialog.Root title="Are you sure you want to delete this app?" bind:open>
    <!-- {#if currentStep == 0} -->
    <p><b>{display_name}</b> will be permanently deleted and cannot be recovered!!</p>
    <!-- {/if} -->

    <!-- {#if currentStep == 1}
        <p>Save your app secret now!! It won't be shown again.</p>
        <span>
            <code>{app_secret}</code>
        </span>
    {/if} -->
    {#snippet actions()}
        <Button
            negative
            shouldFill
            onclick={() => doRevoke()}
            loading={deleteApp.isPending}
            disabled={deleteApp.isPending}>Yes, I am sure</Button
        >
    {/snippet}
</Dialog.Root>
