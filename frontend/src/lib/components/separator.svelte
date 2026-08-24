<script lang="ts">
    type Props = { text?: string };
    let { text }: Props = $props();
</script>

<span class="separator">
    {#if text}{text}{/if}
</span>

<style lang="scss">
    .separator {
        display: flex;
        align-items: center;
        text-align: center;
        font-size: var(--text-small);
        inline-size: 100%;
        gap: 1ch;
        font-weight: 300;

        &::before,
        &::after {
            content: "";
            flex: 1;
            block-size: 1px;
            background-color: var(--color-iron-medium);
        }
    }
</style>
