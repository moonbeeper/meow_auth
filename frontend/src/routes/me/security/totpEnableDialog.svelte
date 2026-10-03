<script lang="ts" module>
    import { z } from "zod";
    const schema = z.object({
        code: z.string().regex(PIN_DIGIT_REGEX, "Invalid code").length(6, "Invalid code")
    });
</script>

<script lang="ts">
    import { goto } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";
    import InputError from "$comps/form/inputError.svelte";
    import Pin from "$comps/form/pin.svelte";
    import { PIN_DIGIT_REGEX } from "$comps/form/regex";
    import LoadingBox from "$comps/loadingBox.svelte";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import type { CreateTotpResponse } from "$lib/api/model";
    import queryClient from "$lib/api/tanstackClient";
    import { createEnableTotpOptions, createExchangeTotpOptions } from "$lib/api/totp/totp";
    import { getCurrentUserInfoQueryKey } from "$lib/api/user/user";
    import { Control, Field } from "formsnap";
    import encodeQR from "qr";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import TotpRecoveryCodes from "./totpRecoveryCodes.svelte";

    let currentStep = $state(0);
    let isDialogOpen = $state(false);

    function nextStep() {
        currentStep = Math.min(currentStep + 1, 2);
    }

    function prevStep() {
        currentStep = Math.max(currentStep - 1, 0);
    }

    function resetDialog() {
        isDialogOpen = false;
        enrollmentData = null;
        currentStep = 0;
        exchangeSuccess = false;
        rawForm.reset();
    }

    const getTotpOptions = createEnableTotpOptions();

    let enrollmentData = $state<CreateTotpResponse | null>(null);
    let qrCodeUrl = $derived.by(() => {
        if (!enrollmentData) return null;
        return encodeQR(enrollmentData.uri, "svg");
    });

    async function firstStep() {
        if (enrollmentData || getTotpOptions.isPending) return;
        const options = await getTotpOptions.mutateAsync();

        if (!isOk(options)) {
            console.error("failed to get totp options");

            if (options.data.code == "SudoNotEnabled") {
                console.error("sudo not enabled, cannot enable totp. redirecting");
                await goto(`/auth/sudo?redirect=${encodeURIComponent("/me/security")}`);
            }

            resetDialog();
            return;
        }
        enrollmentData = options.data;
    }

    const exchangeTotpOptions = createExchangeTotpOptions(() => ({
        mutation: {
            onSuccess: () => {
                queryClient.invalidateQueries({ queryKey: getCurrentUserInfoQueryKey() });
            }
        }
    }));
    let exchangeSuccess = $state(false);

    const rawForm = superForm(defaults(zod4(schema)), {
        SPA: true,
        resetForm: false,
        validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(schema),
        onUpdate: async ({ form }) => {
            if (!form.valid) {
                console.warn("somehow the form was submitted without being valid");
                return;
            }

            const res = await exchangeTotpOptions.mutateAsync({
                data: {
                    code: form.data.code
                }
            });

            if (!isOk(res)) {
                console.error("failed to get totp options");

                if (res.data.code == "SudoNotEnabled") {
                    console.error("sudo not enabled, cannot enable totp. redirecting");
                    await goto(`/auth/sudo?redirect=${encodeURIComponent("/me/security")}`);
                } else if (res.data.code == "InvalidCode") {
                    console.error("invalid code");
                    setError(form, "code", "Invalid code");
                    return;
                } else {
                    console.error("unknown error :(");
                }

                resetDialog();
                return;
            }

            exchangeSuccess = true;
            console.log("done enabling totp");
            resetDialog();
        }
    });

    const { form, enhance, delayed } = rawForm;

    $effect(() => {
        if (!isDialogOpen) {
            resetDialog();
            return;
        }
        if (isDialogOpen && currentStep == 0) {
            firstStep();
        }
    });
</script>

<Dialog.Root title="Enable Two-Factor Authentication" bind:open={isDialogOpen}>
    {#snippet trigger({ props })}
        <Button primary fontSize="small" {...props} disabled={isDialogOpen} loading={isDialogOpen}
            >Set up 2FA</Button
        >
    {/snippet}
    {#if currentStep == 0}
        <p>
            Enabling Two-Factor Authentication (2FA) will need you to use an authenticator app (eg.
            1Password, Authy) to generate a code to log in or enable sudo
        </p>
        {#if enrollmentData}
            <!-- TODO: spinner? -->
            {#if qrCodeUrl}
                <div class="qr-code">{@html qrCodeUrl}</div>
            {/if}

            <p class="secret">
                Can't scan the QR code? Copy the secret:
                <strong class="hint">{enrollmentData.secret}</strong>
            </p>
        {:else}
            <div>
                <LoadingBox
                    what="2FA options"
                    loaded={enrollmentData != null}
                    hasError={getTotpOptions.isError}
                />
            </div>
        {/if}
    {/if}

    {#if currentStep == 1}
        <TotpRecoveryCodes recoveryCodes={enrollmentData?.recovery_codes ?? []} />
    {/if}

    {#if currentStep == 2}
        <p>To finalize, enter one of the generated codes from your authenticator</p>
        <form class="form" method="post" use:enhance>
            <Field form={rawForm} name="code">
                <Control>
                    {#snippet children({ props })}
                        <Pin
                            {...props}
                            loading={$delayed}
                            bind:value={$form.code}
                            onComplete={() => rawForm.submit()}
                        />
                    {/snippet}
                </Control>
                <InputError />
            </Field>
        </form>
    {/if}

    {#snippet actions()}
        {#if currentStep > 0}
            <Button onclick={prevStep} disabled={$delayed || exchangeSuccess}>Go back</Button>
        {/if}

        <Dialog.Close disabled={$delayed || exchangeSuccess}>Cancel</Dialog.Close>

        {#if enrollmentData}
            {#if currentStep == 2}
                <Button
                    primary
                    onclick={() => rawForm.submit()}
                    loading={$delayed}
                    disabled={$delayed || exchangeSuccess}>Finish</Button
                >
            {:else}
                <Button onclick={nextStep} primary>Continue</Button>
            {/if}
        {/if}
    {/snippet}
</Dialog.Root>

<style lang="scss">
    .qr-code {
        background-color: white;
        margin: auto;
        inline-size: 250px;
    }

    .secret {
        overflow-wrap: anywhere;
        word-break: break-word;

        .hint {
            font-family: var(--font-mono);
        }
    }

    .form {
        display: flex;
        flex-direction: column;
        gap: var(--spacing);
    }
</style>
