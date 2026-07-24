<script lang="ts">
    import Pin from "$comps/form/pin.svelte";
    import { PIN_DIGIT_AND_CHAR } from "$comps/form/regex";
</script>

<main id="main" class="grid justify-center margin-auto">
    <div class="authbox">
        <header class="authbox--header">
            <h1 class="authbox--title">Enter your Two-Factor recovery code</h1>
            <p class="authbox--subtitle">
                Use a recovery code saved during Two-Factor setup below:
            </p>
        </header>
        <form action="meow" class="authbox--form" method="post">
            <Pin max_length={10} regex={PIN_DIGIT_AND_CHAR} />
            <p class="font-medium">
                Lost access to your authenticator? <a href="/auth/totp/recovery"
                    >Use a recovery code</a
                >
            </p>
        </form>
    </div>
</main>
