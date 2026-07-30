<script lang="ts">
    import type { Snippet } from "svelte";

    // you cant actually do more snippets. that's not how layouts work, Svelte, like you might have
    // more than one snippet for... ACTIONS or idk man. that's just not I mentally imaged it.
    let {
        children
    }: {
        children: Snippet;
    } = $props();
</script>

<main id="main" class="grid" tabindex="-1">
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
