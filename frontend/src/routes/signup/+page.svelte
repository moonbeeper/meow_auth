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
    import { otpRegister } from "$lib/api/auth/auth";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import { auth } from "$lib/auth/auth.svelte";
    import { Control, Field } from "formsnap";
    import { defaults, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    const rawForm = superForm(defaults(zod4(schema)), {
        SPA: true,
        validators: zod4(schema),
        onUpdate: async ({ form }) => {
            const res = await otpRegister({ email: form.data.email });

            if (!isOk(res)) {
                console.error("i will cry");
                return;
            }
            const { flow_id } = res.data;

            console.log("going otp route");
            auth.pendingAuthEmail = form.data.email;
            await goto(`/auth/${flow_id}/otp`);
            return;
        }
    });
    const { form, enhance, delayed } = rawForm;
</script>

<AuthBox.Root>
    <AuthBox.Header title="Sign up" />
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
        <p class="font-medium">Already have an account? <a href="/">Log in</a>!</p>
        <!-- <button class="button button--primary font-medium">Continue</button> -->
        <Button primary fontSize="medium" type="submit" disabled={$delayed} loading={$delayed}>
            Continue
        </Button>
    </form>
</AuthBox.Root>
