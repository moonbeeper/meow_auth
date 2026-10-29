<script lang="ts">
    import { goto } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";
    import { createDeleteOauthApplication, getListOauthApplicationsQueryKey } from "$lib/api";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import type { OauthApplication } from "$lib/api/model";
    import queryClient from "$lib/api/tanstackClient";

    export type Props = {
        item: OauthApplication;
        open: boolean;
    };

    let { item, open = $bindable(false) }: Props = $props();
    let currentStep = $state(0);

    let deleteApp = createDeleteOauthApplication(() => ({
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
    }

    async function doRevoke() {
        if (!item) {
            return;
        }
        const res = await deleteApp.mutateAsync({
            id: item.id
        });

        if (!isOk(res)) {
            console.error("failed to delete app");
        } else {
            console.info("deleted app " + item.id);
        }

        resetDialog();
        await goto("/me/my-apps");
    }

    let title = $derived.by(() => {
        if (currentStep == 0) {
            return "Are you sure you want to delete this app?";
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
