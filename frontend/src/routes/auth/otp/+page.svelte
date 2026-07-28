<script lang="ts">
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";
    import Pin from "$comps/form/pin.svelte";
    import { PIN_DIGIT_AND_CHAR } from "$comps/form/regex";
</script>

<AuthBox.Root>
    <AuthBox.Header
        title="Check your email!"
        subtitle="There should be a verification code that you input below:"
    />
    <form action="meow" class="authbox--form" method="post">
        <Pin regex={PIN_DIGIT_AND_CHAR} />
    </form>
</AuthBox.Root>
