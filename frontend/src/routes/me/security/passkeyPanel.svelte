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
        createListPasskeys,
        createRegisterPasskeyExchange,
        createRegisterPasskeyOptions,
        getListPasskeysQueryKey
    } from "$lib/api/passkeys/passkeys";
    import queryClient from "$lib/api/tanstackClient";
    import { actionDate, tryCatch } from "$lib/common";
    import type { PublicKeyCredentialCreationOptionsJSON } from "@simplewebauthn/browser";
    import { startRegistration } from "@simplewebauthn/browser";

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

    let isAddingPasskey = $state(false);
    async function handleAddPasskey() {
        if (isAddingPasskey) return; // oh you can do that.
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
            isAddingPasskey = false;
            return;
        }

        // let attestationResult: Awaited<ReturnType<typeof startRegistration>>;
        // try {
        //     attestationResult = await startRegistration({
        //         optionsJSON: options.data.publicKey as PublicKeyCredentialCreationOptionsJSON
        //     });
        // } catch (e) {
        //     console.error("failed to get attestation result: ", e);
        //     isAddingPasskey = false;
        //     return;
        // }
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

        isAddingPasskey = false;
    }
</script>

<SettingsPanel
    title="Passkeys"
    description="Ah, efficient sign in. Modern and more secure!"
    contentSpacing={2}
>
    {#if passkeyList.length != 0}
        <div class="actions">
            <Button
                primary
                onclick={handleAddPasskey}
                disabled={isAddingPasskey}
                loading={isAddingPasskey}>Add Passkey</Button
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
            <LogItem.Container>
                {#each passkeyList as passkey (passkey.id)}
                    <PasskeyItem
                        title={passkey.display_name}
                        when={actionDate(passkey.created_at)}
                    />
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
