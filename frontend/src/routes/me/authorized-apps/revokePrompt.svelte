<script lang="ts">
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";
    import { createRevokeOauthAuthorization, getListOauthAuthorizationsQueryKey } from "$lib/api";
    import type { ListDataResponseOauthAuthorizationDataItem } from "$lib/api/model";
    import queryClient from "$lib/api/tanstackClient";

    export type Props = {
        item: ListDataResponseOauthAuthorizationDataItem | null;
        open: boolean;
    };

    let { item, open = $bindable(false) }: Props = $props();

    const revokeAuthorization = createRevokeOauthAuthorization(() => ({
        mutation: {
            onSuccess: () => {
                queryClient.invalidateQueries({ queryKey: getListOauthAuthorizationsQueryKey() });
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
        await revokeAuthorization.mutateAsync({ id: item.id });
        resetDialog();
    }

    $effect(() => {
        if (!open) {
            resetDialog();
            return;
        }
    });
</script>

<Dialog.Root title="Are you sure you want to revoke this authorization?" bind:open>
    <p><b>{display_name}</b> will no longer have access to your account information!!</p>
    {#snippet actions()}
        <Button
            negative
            shouldFill
            onclick={() => doRevoke()}
            loading={revokeAuthorization.isPending}
            disabled={revokeAuthorization.isPending}>Yes, I am sure</Button
        >
    {/snippet}
</Dialog.Root>
