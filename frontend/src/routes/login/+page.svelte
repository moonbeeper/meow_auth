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
    import { flowOptions, otpLogin } from "$lib/api/auth/auth";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import { auth } from "$lib/auth/auth.svelte";
    import { Control, Field } from "formsnap";
    import { defaults, setMessage, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

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

    const rawForm = superForm(defaults(zod4(schema)), {
        SPA: true,
        // validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(schema),
        onUpdate: async ({ form }) => {
            const res = await flowOptions({ email: form.data.email });

            if (!isOk(res)) {
                console.error("i will cry");
                return;
            }
            const { methods } = res.data;

            if (methods.includes("passkey")) {
                console.log("going webauthn route by priority lol");
                return;
            } else if (methods.includes("otp")) {
                console.log("going otp route");
                const req = await otpLogin({ email: form.data.email });

                if (!isOk(req)) {
                    console.error("otp login faild :(");
                    return;
                }

                auth.pendingAuthEmail = form.data.email;
                await goto(`/auth/${req.data.flow_id}/otp`);
                return;
            } else {
                console.warn("somehow login options returned a non valid method list");
                return;
            }
        }
    });
    const { form, enhance, delayed } = rawForm;
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
                        autocomplete="email"
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
        <!-- <button class="button button--primary font-medium">Continue</button> -->
        <Button primary fontSize="medium" type="submit" disabled={$delayed} loading={$delayed}>
            Continue
        </Button>
    </form>
</AuthBox.Root>
