<script lang="ts">
    import type { Icon } from "@tabler/icons-svelte-runes";
    import type { ComponentProps } from "svelte";

    type SymbolProps = ComponentProps<Icon>;
    type Props = Omit<SymbolProps, "class" | "children"> & {
        symbol: Icon;
        size?: "smol" | "small" | "medium";
    };

    let { symbol: Symbol, stroke = 2, size = "smol", ...props }: Props = $props();
</script>

<span class={["icon", size]}>
    <Symbol {stroke} {...props} />
</span>

<style lang="scss">
    .icon {
        --size-multiplier: 1;

        position: relative;
        display: inline-flex;
        flex-shrink: 0;
        align-items: center;
        justify-content: center;
        color: inherit;
        vertical-align: middle;

        inline-size: 1em;
        block-size: 1em;
        // transform: translateY(-0.05em);

        :global(svg) {
            position: absolute;
            // inset: 0;
            margin: auto;
            display: block;
            block-size: calc(1em * var(--size-multiplier));
            inline-size: calc(1em * var(--size-multiplier));
        }
    }

    .small {
        --size-multiplier: 1.25;
    }

    .medium {
        --size-multiplier: 1.5;
    }
</style>
