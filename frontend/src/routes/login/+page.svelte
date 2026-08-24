<script lang="ts" module>
    import { z } from "zod";

    const schema = z.object({
        email: z.email()
    });
</script>

<script lang="ts">
    import { goto } from "$app/navigation";
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";
    import Input from "$comps/form/input.svelte";
    import InputError from "$comps/form/inputError.svelte";
    import Separator from "$comps/separator.svelte";
    import { flowOtpStart, flowWebauthnExchange, flowWebauthnStart } from "$lib/api/auth/auth";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import { auth } from "$lib/auth/auth.svelte";
    import { tryCatch } from "$lib/common";
    import {
        startAuthentication,
        WebAuthnAbortService,
        WebAuthnError,
        type PublicKeyCredentialRequestOptionsJSON
    } from "@simplewebauthn/browser";
    import { Control, Field } from "formsnap";
    import { onMount } from "svelte";
    import { defaults, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();

    const greetings = [
        "Meow!",
        "Psst, over here!",
        "Another wanderer...",
        "You rang?",
        "Greetings, traveler",
        "Oh, it's you"
    ];

    let greeting = $derived.by(() => {
        return greetings[Math.floor(Math.random() * greetings.length)];
    });

    let redirectTo = $derived.by(() => {
        if (data.redirect) {
            return `?redirect=${encodeURIComponent(data.redirect)}`;
        }
        return "";
    });

    const rawForm = superForm(defaults(zod4(schema)), {
        SPA: true,
        validators: zod4(schema),
        onUpdate: async ({ form }) => {
            // kill the passkey flow if the user is trying to use email instead
            WebAuthnAbortService.cancelCeremony();
            auth.pendingAuthEmail = form.data.email;

            const res = await tryCatch(() => flowOtpStart({ email: form.data.email }));

            if (res.error || !isOk(res.result)) {
                console.error("failed to create otp flow: ", res.error);
                return;
            }
            const { flow_id } = res.result.data;

            await goto(`/auth/${flow_id}/otp${redirectTo}`);
            return;
        }
    });

    const { form, enhance, delayed } = rawForm;

    type PasskeyError = {
        type: "error" | "cancel";
        message: string;
    };
    let passkeyPending = $state(false);
    let passkeyError = $state<PasskeyError | null>(null);

    function setPasskeyState(pending: boolean = true, error: PasskeyError | null = null) {
        passkeyPending = pending;
        passkeyError = error;
    }

    async function startPasskeyFlow() {
        if (passkeyPending) return;
        setPasskeyState();
        const res = await tryCatch(() => flowWebauthnStart());
        if (res.error || !isOk(res.result)) {
            console.error("failed to start passkey flow", res.error);

            if (res.result?.data && "code" in res.result.data) {
                if (res.result.data.code == "RatelimitExceeded") {
                    setPasskeyState(false, {
                        type: "error",
                        message: "You have exceeded the rate limit for passkey logins! Wait a bit"
                    });
                    return;
                }
            }

            setPasskeyState(false, {
                type: "error",
                message: "Server did not respond with challenge"
            });
            return;
        }

        const { publicKey } = res.result.data;

        const attestationResult = await tryCatch(() =>
            startAuthentication({
                optionsJSON: publicKey as PublicKeyCredentialRequestOptionsJSON,
                useBrowserAutofill: true
            })
        );

        if (attestationResult.error) {
            const error = attestationResult.error;

            if (error instanceof WebAuthnError && error.name == "NotAllowedError") {
                console.warn("user cancelled passkey prompt or has expired");
                setPasskeyState(false, {
                    type: "cancel",
                    message: "You cancelled the passkey prompt or it has expired"
                });
                return;
            }

            console.error("failed to get attestation result: ", attestationResult.error);
            setPasskeyState(false, {
                type: "error",
                message: "Something went wrong while trying to use your passkey"
            });
            return;
        }

        const exchangeRes = await tryCatch(() =>
            flowWebauthnExchange({
                id: attestationResult.result.id,
                rawId: attestationResult.result.rawId,
                response: attestationResult.result.response,
                type: attestationResult.result.type,
                extensions: attestationResult.result.clientExtensionResults
            })
        );

        if (exchangeRes.error || !isOk(exchangeRes.result)) {
            console.error("failed to exchange passkey", exchangeRes.error);
            setPasskeyState(false, {
                type: "error",
                message: "Something went wrong while trying to use your passkey"
            });
            return;
        }

        setPasskeyState(false);
        console.log("successfully logged in with passkey, redirecting");
        await goto(data.redirect ?? "/me");
    }

    let passkeyErrorClass = $derived.by(() => {
        if (!passkeyError) return "";
        return passkeyError.type == "error" ? "error" : "cancel";
    });

    onMount(() => {
        startPasskeyFlow();
    });
</script>

<AuthBox.Root>
    <AuthBox.Header title={greeting} />
    <form class="form" use:enhance method="post">
        <Field form={rawForm} name="email">
            <Control>
                {#snippet children({ props })}
                    <Input
                        type="email"
                        fontSize="large"
                        placeholder="Your email address"
                        autocomplete="email webauthn"
                        disabled={$delayed}
                        required
                        {...props}
                        bind:value={$form.email}
                    />
                {/snippet}
            </Control>
            <InputError />
        </Field>
        <p class="font-medium">New around this auth realm? <a href="/signup">Sign up</a>!</p>
        <Button primary fontSize="medium" type="submit" disabled={$delayed} loading={$delayed}>
            Continue
        </Button>
    </form>
    <Separator text="or" />
    <div class="passkey">
        <div>
            <Button onclick={() => startPasskeyFlow()}>Use a Passkey</Button>
        </div>
        {#if passkeyError}
            <p class={["hint", passkeyErrorClass]}>{passkeyError.message}</p>
        {/if}
    </div>
</AuthBox.Root>

<style lang="scss">
    .passkey {
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * 2);
    }

    .hint {
        --text-color: inherit;
        color: var(--text-color);
        font-size: var(--text-small);
        font-weight: 500;
        &.error {
            --text-color: var(--color-coral-medium);
            @media (prefers-color-scheme: dark) {
                ----text-color: var(--color-coral-light);
            }
        }
    }
</style>
