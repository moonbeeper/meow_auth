<script lang="ts">
    import { goto } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import EmptyBox from "$comps/emptyBox.svelte";
    import PasskeyItem from "$comps/items/passkeyItem.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import {
        createDeletePasskey,
        createListPasskeys,
        createRegisterPasskeyExchange,
        createRegisterPasskeyOptions,
        getListPasskeysQueryKey
    } from "$lib/api/passkeys/passkeys";
    import queryClient from "$lib/api/tanstackClient";
    import { actionDate, tryCatch } from "$lib/common";
    import type { PublicKeyCredentialCreationOptionsJSON } from "@simplewebauthn/browser";
    import { startRegistration } from "@simplewebauthn/browser";

    import PasskeyNewRenameDialog, { type ActionMode } from "./passkeyNewRenameDialog.svelte";

    const passkeysQuery = createListPasskeys();

    const passkeyList = $derived.by(() => {
        if (passkeysQuery.data?.status != 200) return [];
        return passkeysQuery.data.data ?? [];
    });

    const getAddPasskeyOptions = createRegisterPasskeyOptions();
    const addPasskey = createRegisterPasskeyExchange(() => ({
        mutation: {
            onSuccess: () => {
                console.log("successfully added passkey");
                queryClient.invalidateQueries({ queryKey: getListPasskeysQueryKey() });
            }
        }
    }));

    let actionMode = $state<ActionMode>("add");
    let isRenaming = $state(false);
    let renameCurrentName = $state<string | undefined>(undefined);
    let isAddingPasskey = $state(false);
    let isNamePromptOpen = $state(false);
    let isNamePromptReady = $state(false);
    let passkeyIdToRename = $state<string>("01M3TQNZXMZEK2DJHDXPKMSCQW"); // placeholder ulid

    let userCancelledAttestation = $state(false);
    let userFinishedAttestation = $state(false);
    async function handleAddPasskey() {
        if (isAddingPasskey) return; // oh you can do that.
        actionMode = "add";
        console.log("going to add passkey");

        const options = await getAddPasskeyOptions.mutateAsync();
        if (!isOk(options)) {
            if (options.data.code == "SudoNotEnabled") {
                console.error("sudo not enabled, cannot add passkey. redirecting");
                await goto(`/auth/sudo?redirect=${encodeURIComponent("/me/security")}`);
            }
            //  TODO: should have a dialog or toast
            console.error("failed to get passkey options");
            return;
        }
        isAddingPasskey = true;
        console.log("got these optiosn: ", options);

        let attestationResult = await tryCatch(() =>
            startRegistration({
                optionsJSON: options.data.publicKey as PublicKeyCredentialCreationOptionsJSON
            })
        );

        if (attestationResult.error) {
            console.error("failed to get attestation result: ", attestationResult.error);

            if (attestationResult.error.name == "NotAllowedError") {
                console.warn("user cancelled the attestation");
                userCancelledAttestation = true;
            }
            isAddingPasskey = false;
            return;
        }

        console.log("got attestation result: ", attestationResult);

        const res = await addPasskey.mutateAsync({
            data: {
                extensions: attestationResult.result.clientExtensionResults,
                id: attestationResult.result.id,
                rawId: attestationResult.result.rawId,
                response: attestationResult.result.response,
                type: attestationResult.result.type
            }
        });

        if (!isOk(res)) {
            console.error("failed to add passkey");
            isAddingPasskey = false;
            return;
        }

        userFinishedAttestation = true;
        passkeyIdToRename = res.data.id;
        isNamePromptReady = true;
        isNamePromptOpen = true;
        isAddingPasskey = false;
    }

    let buttonText = $derived.by(() => {
        if (isAddingPasskey) return "Adding Passkey";
        if (userCancelledAttestation) return "Creation cancelled";
        if (userFinishedAttestation) return "Creation finished";
        return "Add Passkey";
    });

    const requestDeletePasskey = createDeletePasskey(() => ({
        mutation: {
            onSuccess: () => {
                queryClient.invalidateQueries({ queryKey: getListPasskeysQueryKey() });
            }
        }
    }));

    async function handleOnDelete(id: string) {
        const res = await requestDeletePasskey.mutateAsync({
            id: id
        });

        if (!isOk(res)) {
            if (res.data.code == "SudoNotEnabled") {
                console.error("sudo not enabled, cannot delete passkey. redirecting");
                await goto(`/auth/sudo?redirect=${encodeURIComponent("/me/security")}`);
            }

            console.error("failed to delete passkey");
            return;
        }
        console.info("deleted passkey " + id);
    }

    async function handleOnRename(id: string, name: string) {
        actionMode = "update";
        renameCurrentName = name;
        passkeyIdToRename = id;
        isRenaming = true;
        isNamePromptReady = true;
        isNamePromptOpen = true;
    }

    let actionBool = $derived.by(() => {
        if (actionMode == "update") {
            return isRenaming;
        }
        return isAddingPasskey;
    });

    $effect(() => {
        if (!isNamePromptOpen) {
            isAddingPasskey = false;
            isRenaming = false;
        }
    });

    $effect(() => {
        if (userCancelledAttestation) {
            setTimeout(() => {
                userCancelledAttestation = false;
            }, 1000);
        }

        if (userFinishedAttestation) {
            setTimeout(() => {
                userFinishedAttestation = false;
            }, 1000);
        }
    });
</script>

<SettingsPanel
    title="Passkeys"
    description="Ah, efficient sign in. Modern and more secure!"
    contentSpacing={2}
>
    <PasskeyNewRenameDialog
        id={passkeyIdToRename}
        bind:open={isNamePromptOpen}
        bind:actionBool
        isReady={isNamePromptReady}
        {actionMode}
        currentName={renameCurrentName}
    />

    {#if passkeyList.length != 0}
        <div class="actions">
            <Button
                primary={!userCancelledAttestation}
                negative={userCancelledAttestation}
                gooder={userFinishedAttestation}
                onclick={handleAddPasskey}
                disabled={isAddingPasskey}
                loading={isAddingPasskey}>{buttonText}</Button
            >
        </div>
    {/if}

    {#if passkeysQuery.isLoading || passkeysQuery.isError}
        <LoadingBox
            what="passkeys"
            loaded={!passkeysQuery.isLoading}
            hasError={passkeysQuery.isError}
        />
    {:else if passkeysQuery.isSuccess}
        {#if passkeyList.length != 0}
            <LogItem.Container collapsible>
                {#each passkeyList as passkey (passkey.id)}
                    <PasskeyItem onDelete={handleOnDelete} onRename={handleOnRename} {passkey} />
                {/each}
            </LogItem.Container>
        {:else}
            <EmptyBox
                title="No passkeys yet"
                subtitle="Convenient, secure, fast... sign in with a tap and you're in!! GOOODbye email codes, hello biometric (or physical) magic!"
            >
                {#snippet actions()}
                    <Button
                        primary
                        fontSize="small"
                        onclick={handleAddPasskey}
                        disabled={isAddingPasskey}
                        loading={isAddingPasskey}
                    >
                        Add Passkey
                    </Button>
                {/snippet}
            </EmptyBox>
        {/if}
    {/if}
</SettingsPanel>

<style lang="scss">
    .actions {
        display: flex;
        inline-size: 100%;
        justify-content: end;
        align-items: center;
    }
</style>
