<script lang="ts">
    import type { Snippet } from "svelte";

    let { children }: { children: Snippet } = $props();
</script>

<main id="main" class="grid justify-center margin-auto" tabindex="-1">
    <div class="authbox">
        {@render children()}
    </div>
</main>

<style lang="scss">
    .authbox {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: calc(var(--spacing) * 2);
        // adjusted again because BIG mode on the pin component made so the 2fa recovery input (which has 10 digits)
        // be too wide, overflowing and having to scroll. i dont like that :(
        max-inline-size: 38ch;
    }

    .authbox :global(.form) {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: calc(var(--spacing) * 3);
        // max-inline-size: 35ch;
        // inline-size: 100%;
    }
</style>
