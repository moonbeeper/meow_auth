<script lang="ts" module>
    import { z } from "zod";
    const schema = z.object({
        code: z.string().regex(PIN_DIGIT_AND_CHAR, "Invalid code").length(6, "Invalid code")
    });
</script>

<script lang="ts">
    import { goto, invalidateAll } from "$app/navigation";
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";
    import InputError from "$comps/form/inputError.svelte";
    import Pin from "$comps/form/pin.svelte";
    import { PIN_DIGIT_AND_CHAR } from "$comps/form/regex";
    import { flowOtpExchange, otpLogin } from "$lib/api/auth/auth";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import { auth } from "$lib/auth/auth.svelte";
    import { error } from "@sveltejs/kit";
    import { Control, Field } from "formsnap";
    import { fly } from "svelte/transition";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();
    let resendTimeout = $state(false);
    let resendLoading = $state(false);

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

            const res = await flowOtpExchange({ flow_id: data.flowId, code: form.data.code });

            if (!isOk(res)) {
                // if (res.data.code == "RatelimitExceeded") { i have to find another way, i dont like this one
                //     console.error("auth ratelimit exceeded");
                //     setError(form, "code", "Ratelimit exceeded, please try again later");
                // }
                console.warn("invalid code submitted");
                setError(form, "code", "Invalid code");
                return;
            }

            if ("next_method" in res.data) {
                let next_method = res.data.next_method;
                console.log("next auth method is: ", next_method);

                if (next_method.includes("otp")) {
                    console.error("the next auth method cannot be once again otp");
                }

                await goto(`/auth/${res.data.flow_id}/totp`);
                return;
            }

            console.log("finalized authentication");
            await invalidateAll();
            await goto("/");
        }
    });

    const { form, enhance, delayed } = rawForm;

    async function resendCode() {
        if (resendLoading) {
            console.warn("already resending code, ignoring");
            return;
        }

        console.log("resending code and redirecting");
        if (!auth.pendingAuthEmail) {
            console.error("no pending auth email, cannot resend code");
            await goto("/");
        }
        resendLoading = true;
        try {
            const req = await otpLogin({ email: auth.pendingAuthEmail as string });

            if (!isOk(req)) {
                await goto("/");
                console.error("resending otp login faild :(");
                return;
            }
            await goto(`/auth/${req.data.flow_id}/otp`);
        } finally {
            resendLoading = false;
        }
    }

    $effect(() => {
        let _ = data.flowId; // making svelte trigger this when the flow id changes (when resending the code!!)
        resendTimeout = false;
        const timer = setTimeout(
            () => {
                resendTimeout = true;
            },
            1000 * 60 * 4 // 4 minutes, 1 minute less than the server timeout of 5 minutes.
        );
        return () => clearTimeout(timer);
    });
</script>

<AuthBox.Root>
    <AuthBox.Header
        title="Check your email!"
        subtitle="There should be a verification code that you input below:"
    />
    <form class="form" method="post" use:enhance>
        <Field form={rawForm} name="code">
            <Control>
                {#snippet children({ props })}
                    <Pin
                        regex={PIN_DIGIT_AND_CHAR}
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
    {#if resendTimeout}
        <div class="send-again" transition:fly={{ y: 20 }}>
            <p>Is the code not working?</p>
            <Button onclick={resendCode} disabled={resendLoading} loading={resendLoading}
                >Send again!</Button
            >
        </div>
    {/if}
</AuthBox.Root>

<style lang="scss">
    .send-again {
        display: flex;
        align-items: center;
        gap: calc(var(--spacing) * 2);

        @media (max-width: 321px) {
            flex-direction: column;
        }
    }
</style>
