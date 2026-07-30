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
        max-inline-size: 35ch;
    }
</style>
