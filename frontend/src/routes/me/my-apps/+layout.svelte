<script lang="ts">
    import type { Snippet } from "svelte";
    let {
        children
    }: {
        children: Snippet;
    } = $props();
</script>

<svelte:head>
    <title>My Apps - Meow Auth</title>
</svelte:head>

{@render children()}
