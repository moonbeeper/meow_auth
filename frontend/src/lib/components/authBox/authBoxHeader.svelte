<script lang="ts">
    let { title, subtitle }: { title?: string; subtitle?: string } = $props();
</script>

<header class="header">
    <h1 class="title">{title}</h1>
    {#if subtitle}
        <p class="subtitle">{subtitle}</p>
    {/if}
</header>

<style lang="scss">
    .header {
        text-align: center;
        margin-block-end: calc(var(--spacing) * 4);
    }

    .title {
        font-size: var(--text-larger);
    }

    .subtitle {
        font-size: var(--text-medium);
    }
</style>
