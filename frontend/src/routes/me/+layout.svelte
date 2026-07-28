<script lang="ts">
    import type { Snippet } from "svelte";

    let {
        children
    }: {
        children: Snippet;
    } = $props();
</script>

<main id="main" class="grid">
    <div class="settings">
        {@render children()}
    </div>
</main>

<style lang="scss">
    .settings {
        max-inline-size: 72rem; // 1152px
        inline-size: 100%; // without this, it wont do anything the above.
        margin-inline: auto;
    }
</style>
