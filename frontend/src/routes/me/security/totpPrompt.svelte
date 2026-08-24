<script lang="ts" module>
    import { z } from "zod";

    const codeSchema = z.object({
        code: z.string().regex(PIN_DIGIT_REGEX, "Invalid code").length(6, "Invalid code")
    });

    const recoverySchema = z.object({
        code: z.string().regex(PIN_DIGIT_AND_CHAR, "Invalid code").length(10, "Invalid code")
    });
</script>

<script lang="ts">
    import { goto } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";
    import InputError from "$comps/form/inputError.svelte";
    import Pin from "$comps/form/pin.svelte";
    import { PIN_DIGIT_AND_CHAR, PIN_DIGIT_REGEX } from "$comps/form/regex";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import queryClient from "$lib/api/tanstackClient";
    import { createDisableTotp, createViewTotpRecoveryCodes } from "$lib/api/totp/totp";
    import { getCurrentUserInfoQueryKey } from "$lib/api/user/user";
    import { Control, Field } from "formsnap";
    import { defaults, setError, superForm, type SuperValidated } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import TotpRecoveryCodes from "./totpRecoveryCodes.svelte";

    export type Props = {
        promptType: "disable" | "viewRecoveryCodes";
        open: boolean;
    };

    let { promptType, open = $bindable(false) }: Props = $props();

    let exchangeSuccess = $state(false);
    let currentStep = $state(0);
    let authenticationType = $state<"normal" | "recovery">("normal");
    let recoveryCodes = $state<string[] | undefined>(undefined);

    const boilerplate = {
        mutation: {
            onSuccess: () => {
                queryClient.invalidateQueries({ queryKey: getCurrentUserInfoQueryKey() });
            }
        }
    };
    const disableTotp = createDisableTotp(() => boilerplate);
    const viewRecoveryCodes = createViewTotpRecoveryCodes(() => boilerplate);

    let queryMethod = $derived.by(() => {
        if (promptType == "disable") {
            return disableTotp;
        } else {
            return viewRecoveryCodes;
        }
    });

    let dialogTitle = $derived.by(() => {
        if (promptType == "disable") {
            return "Disable Two-Factor Authentication";
        } else {
            return "View Recovery Codes";
        }
    });

    let closeButtonText = $derived.by(() => {
        if (promptType == "viewRecoveryCodes" && currentStep == 1) {
            return "Close";
        } else {
            return "Cancel";
        }
    });

    function resetDialog() {
        open = false;
        currentStep = 0;
        authenticationType = "normal";
        rawCodeForm.reset();
        rawRecoveryForm.reset();
        exchangeSuccess = false;
    }

    // copy-pasted babyy... it idint like using typeof rawBLAH because the type didnt have some things i used (.valid lol)
    async function submitForm(
        form: SuperValidated<
            {
                code: string;
            },
            any,
            {
                code: string;
            }
        >
    ) {
        if (!form.valid) {
            console.warn("somehow the form was submitted without being valid");
            return;
        }

        const res = await queryMethod.mutateAsync({
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

        if (promptType == "viewRecoveryCodes" && res.data && "recovery_codes" in res.data) {
            recoveryCodes = res.data.recovery_codes;
            currentStep = 1;
            return;
        }
        exchangeSuccess = true;

        resetDialog();
    }

    function onRecoveryPaste(s: string) {
        console.log(s);
        return s.replaceAll("-", "").toUpperCase();
    }

    const rawCodeForm = superForm(defaults(zod4(codeSchema)), {
        SPA: true,
        resetForm: false,
        validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(codeSchema),
        onUpdate: async ({ form }) => await submitForm(form),
        id: "codeForm"
    });

    const rawRecoveryForm = superForm(defaults(zod4(recoverySchema)), {
        SPA: true,
        resetForm: false,
        validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(recoverySchema),
        onUpdate: async ({ form }) => await submitForm(form),
        id: "recoveryForm"
    });

    const { form: codeForm, enhance: enhanceCode, delayed: delayedCode } = rawCodeForm;
    const {
        form: recoveryForm,
        enhance: enhanceRecovery,
        delayed: delayedRecovery
    } = rawRecoveryForm;

    let formSubmit = $derived.by(() => {
        if (authenticationType == "normal") {
            return rawCodeForm.submit;
        } else {
            return rawRecoveryForm.submit;
        }
    });

    $effect(() => {
        if (!open) {
            resetDialog();
            return;
        }
    });
</script>

<Dialog.Root title={dialogTitle} bind:open>
    <!-- {#snippet trigger({ props })}
        <Button primary fontSize="small" {...props}>Set up 2FA</Button>
    {/snippet} -->

    {#if currentStep == 0}
        <p>To finalize, enter one of the generated codes from your authenticator</p>
        {#if authenticationType == "normal"}
            <form class="form" method="post" use:enhanceCode>
                <Field form={rawCodeForm} name="code">
                    <Control>
                        {#snippet children({ props })}
                            <Pin
                                {...props}
                                loading={$delayedCode}
                                bind:value={$codeForm.code}
                                onComplete={() => rawCodeForm.submit()}
                            />
                        {/snippet}
                    </Control>
                    <InputError />
                </Field>
            </form>

            {#if promptType == "disable"}
                <p class="hint">
                    Lost access to your authenticator? <Button
                        fontSize="small"
                        onclick={() => (authenticationType = "recovery")}
                        >Use a recovery code</Button
                    >
                </p>
            {/if}
        {:else}
            <form class="form" method="post" use:enhanceRecovery>
                <Field form={rawRecoveryForm} name="code">
                    <Control>
                        {#snippet children({ props })}
                            <Pin
                                maxLength={10}
                                regex={PIN_DIGIT_AND_CHAR}
                                loading={$delayedRecovery}
                                bind:value={$recoveryForm.code}
                                onComplete={() => rawRecoveryForm.submit()}
                                onPaste={onRecoveryPaste}
                                {...props}
                            />
                        {/snippet}
                    </Control>
                    <InputError />
                </Field>
            </form>

            <!-- shouldnt need gate because you cant access this type without the right promptType -->
            <p class="hint">
                Got back access to your authenticator? <Button
                    fontSize="small"
                    onclick={() => (authenticationType = "normal")}>Use a normal code</Button
                >
            </p>
        {/if}
    {/if}

    {#if currentStep == 1}
        <TotpRecoveryCodes {recoveryCodes} />
    {/if}

    {#snippet actions()}
        <Dialog.Close
            disabled={$delayedCode || $delayedRecovery || exchangeSuccess}
            primary={currentStep == 1}>{closeButtonText}</Dialog.Close
        >

        {#if currentStep == 0}
            <Button
                primary
                onclick={() => formSubmit()}
                loading={$delayedCode || $delayedRecovery}
                disabled={$delayedCode || exchangeSuccess}>Finish</Button
            >
        {/if}
    {/snippet}
</Dialog.Root>

<style lang="scss">
    .form {
        display: flex;
        flex-direction: column;
        gap: var(--spacing);
    }

    .hint {
        display: flex;
        gap: calc(var(--spacing) * 2);
        align-items: center;
        margin-inline: auto;
        justify-content: center;
        flex-wrap: wrap;
        text-align: center;
    }
</style>
