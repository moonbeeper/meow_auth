<script lang="ts" module>
    import { z } from "zod";
    const schema = z.object({
        code: z.string().regex(PIN_DIGIT_AND_CHAR, "Invalid code").length(10, "Invalid code")
    });
</script>

<script lang="ts">
    import { goto, invalidateAll } from "$app/navigation";
    import * as AuthBox from "$comps/authBox";
    import InputError from "$comps/form/inputError.svelte";
    import Pin from "$comps/form/pin.svelte";
    import { PIN_DIGIT_AND_CHAR } from "$comps/form/regex";
    import { flowOtpExchange, flowTotpExchange } from "$lib/api/auth/auth";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import { auth } from "$lib/auth/auth.svelte";
    import { error } from "@sveltejs/kit";
    import { Control, Field } from "formsnap";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();
    let totpRedirect = $derived.by(() => {
        return `/auth/${data.flowId}/totp`;
    });

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

            const res = await flowTotpExchange({ flow_id: data.flowId, code: form.data.code });

            if (!isOk(res)) {
                if (res.data.code == "FlowNotFound") {
                    console.error("flow not found, redirecting to login");
                    // TODO: Id like this to be a dialog!!
                    setError(form, "code", "Flow not found, redirecting to login");
                    setTimeout(() => {
                        console.log("redirecting to login");
                        goto("/");
                    }, 2000);
                    return;
                }

                console.warn("already used or invalid code submitted");
                setError(form, "code", "You've already used this code");
                return;
            }
            console.log("finalized authentication");
            await invalidateAll();
            await goto("/me");
        }
    });
    const { form, enhance, delayed } = rawForm;

    function onPaste(s: string) {
        console.log(s);
        return s.replaceAll("-", "").toUpperCase();
    }
</script>

<AuthBox.Root>
    <AuthBox.Header
        title="Enter your Two-Factor recovery code"
        subtitle="Use a recovery code saved during Two-Factor setup below:"
    />
    <form class="form" method="post" use:enhance>
        <Field form={rawForm} name="code">
            <Control>
                {#snippet children({ props })}
                    <Pin
                        maxLength={10}
                        regex={PIN_DIGIT_AND_CHAR}
                        {...props}
                        {onPaste}
                        loading={$delayed}
                        bind:value={$form.code}
                        onComplete={() => rawForm.submit()}
                    />
                {/snippet}
            </Control>
            <InputError />
        </Field>
        <p class="font-medium">
            Got back access to your authenticator? <a href={totpRedirect}>Use a normal code</a>
        </p>
    </form>
</AuthBox.Root>
