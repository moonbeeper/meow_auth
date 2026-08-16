<script lang="ts">
    import { goto } from "$app/navigation";
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import { sudoOtpStart, sudoWebauthnExchange, sudoWebauthnStart } from "$lib/api/sudo/sudo";
    import { tryCatch } from "$lib/common";
    import {
        startAuthentication,
        type PublicKeyCredentialRequestOptionsJSON
    } from "@simplewebauthn/browser";

    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();

    const greetings = [
        "Hmmmmm... you look familiar",
        "Quick identity check!",
        "Sorry, I don't recognize you",
        "One last little hurdle!",
        "Prove I can trust you",
        "Time for a quick check!",
        "Ah, the security dance",
        "One more thing...",
        "*watches suspiciously*"
    ];

    let greeting = $derived.by(() => {
        return greetings[Math.floor(Math.random() * greetings.length)];
    });

    let redirectTo = $derived.by(() => {
        if (data.redirect) {
            return `&redirect=${encodeURIComponent(data.redirect)}`;
        }
        return "";
    });

    async function handlePasskey() {
        console.log("going passkey route");
        const otpData = await sudoWebauthnStart();
        if (!isOk(otpData)) {
            if (otpData.data.code == "SudoAlreadyEnabled") {
                console.warn("sudo was already enabled");
                await goto(data.redirect ?? "/me");
            } else if (otpData.data.code == "SudoOptionNotAvailable") {
                console.warn("sudo option not available");
                await goto(data.redirect ?? "/me");
            }

            console.error("could not create otp flow");
            await goto(data.redirect ?? "/me");
            return;
        }

        let attestationResult = await tryCatch(() =>
            startAuthentication({
                optionsJSON: otpData.data.publicKey as PublicKeyCredentialRequestOptionsJSON
            })
        );

        if (attestationResult.error) {
            console.error("failed to get attestation result: ", attestationResult.error);
            return;
        }

        console.log("got attestation result: ", attestationResult);

        const res = await sudoWebauthnExchange({
            id: attestationResult.result.id,
            rawId: attestationResult.result.rawId,
            response: attestationResult.result.response,
            type: attestationResult.result.type,
            extensions: attestationResult.result.clientExtensionResults
        });

        if (!isOk(res)) {
            console.error("failed to exchange passkey");
            return;
        }

        console.log("successfully enabled sudo, redirecting");
        await goto(data.redirect ?? "/me");
    }

    async function handleOtp() {
        console.log("going otp route");
        const otpData = await sudoOtpStart();
        if (!isOk(otpData)) {
            if (otpData.data.code == "SudoAlreadyEnabled") {
                console.warn("sudo was already enabled");
                await goto(data.redirect ?? "/me");
            } else if (otpData.data.code == "SudoOptionNotAvailable") {
                console.warn("sudo option not available");
                await goto(data.redirect ?? "/me");
            }

            console.error("could not create otp flow");
            await goto(data.redirect ?? "/me");
            return;
        }
        await goto(`/auth/${otpData.data.flow_id}/otp?sudo=true${redirectTo}`);
    }

    async function handleTotp() {
        console.log("going totp route");
    }
</script>

<AuthBox.Root>
    <AuthBox.Header title={greeting} subtitle="Pick how you'd like to prove it's you" />
    {#if data.options.includes("passkey")}
        <Button primary fontSize="medium" onclick={handlePasskey}>Use a Passkey</Button>
    {/if}
    {#if data.options.includes("totp")}
        <Button primary fontSize="medium" onclick={handleTotp}>Use a 2FA Code</Button>
    {/if}
    {#if data.options.includes("otp")}
        <Button fontSize="medium" onclick={handleOtp}>Use an Email Code</Button>
    {/if}
</AuthBox.Root>
