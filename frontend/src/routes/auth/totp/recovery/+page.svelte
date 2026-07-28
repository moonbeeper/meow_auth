<script lang="ts">
    import * as AuthBox from "$comps/authBox";
    import Pin from "$comps/form/pin.svelte";
    import { PIN_DIGIT_AND_CHAR } from "$comps/form/regex";
</script>

<AuthBox.Root>
    <AuthBox.Header
        title="Enter your Two-Factor recovery code"
        subtitle="Use a recovery code saved during Two-Factor setup below:"
    />
    <form action="meow" class="authbox--form" method="post">
        <Pin max_length={10} regex={PIN_DIGIT_AND_CHAR} />
        <p class="font-medium">
            Lost access to your authenticator? <a href="/auth/totp/recovery">Use a recovery code</a>
        </p>
    </form>
</AuthBox.Root>
