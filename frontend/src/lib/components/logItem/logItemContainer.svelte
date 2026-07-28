<script lang="ts">
    import type { Snippet } from "svelte";

    let { children }: { children: Snippet } = $props();
</script>

<div class="items">
    {@render children()}
</div>

<style lang="scss">
    .items {
        border-radius: var(--typical-radius);
        border: 1px solid var(--color-iron-dark);
        overflow: hidden;
    }
</style>
