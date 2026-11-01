<script lang="ts">
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";

    import {
        createAdminDeleteUserOauthAuthorization,
        createRevokeOauthAuthorization,
        getListOauthAuthorizationsQueryKey
    } from "#lib/api/index.js";
    import type { ListDataResponseOauthAuthorizationDataItem, User } from "#lib/api/model/index.js";
    import queryClient from "#lib/api/tanstackClient.js";

    export type Props = {
        item: ListDataResponseOauthAuthorizationDataItem | null;
        user: User;
        open: boolean;
    };

    let { item, user, open = $bindable(false) }: Props = $props();

    const revokeAuthorization = createAdminDeleteUserOauthAuthorization(() => ({
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
        await revokeAuthorization.mutateAsync({ id: user.id, cid: item.id });
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
    <p><b>{display_name}</b> will no longer have access to <b>{user.login}</b> information!!</p>
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
