<script lang="ts">
    // TODO: i dont like the red colors ive chosen :(
    import type { Snippet } from "svelte";

    type Props = {
        children: Snippet;
        title?: string;
        description?: string;
        negative?: boolean;
        gooder?: boolean;
        contentSpacing?: number;
        headerChildren?: Snippet;
    };
    let {
        children,
        title = "Section Title",
        description = "i love space birbs. and hate space cats. i love space birbs (once again).",
        negative = false,
        gooder = false,
        contentSpacing,
        headerChildren
    }: Props = $props();
</script>

<section class={["panel", { negative, gooder }]} style:--panel-content-spacing={contentSpacing}>
    <header class="header">
        {#if headerChildren}
            {@render headerChildren()}
        {:else}
            <h2>{title}</h2>
            <p>{description}</p>
        {/if}
    </header>
    <div class="content">
        {@render children()}
    </div>
</section>

<style lang="scss">
    .panel {
        --panel-content-spacing: 4;
        display: grid;
        inline-size: 100%;
        // was 17.5rem the first column (header). Has a third column to make the middle column tinier
        // but filling the whole space still
        grid-template-columns: 0.6fr 1fr 0.4fr;
        gap: calc(var(--spacing) * 12);
        padding-block: calc(var(--spacing) * 8);
        border-block-end: 2px dashed var(--color-iron-lighter);

        &:first-child {
            padding-block-start: calc(var(--spacing) * 5);
        }

        &:last-child {
            border-block-end: 0;
        }

        &:only-child {
            --first-padding-start: calc(var(--spacing) * 0);
        }

        @media (max-width: 768px) {
            grid-template-columns: 1fr;
            gap: calc(var(--spacing) * 4);
        }
    }

    .header {
        display: flex;
        flex-direction: column;
        text-align: start;

        h2 {
            font-size: var(--text-medium);

            .panel.negative & {
                color: var(--color-coral-medium);

                @media (prefers-color-scheme: dark) {
                    color: var(--color-coral-light);
                }
            }

            .panel.gooder & {
                color: var(--color-emerald-medium);

                @media (prefers-color-scheme: dark) {
                    color: var(--color-emerald-light);
                }
            }
        }

        p {
            font-size: var(--text-small);
            color: var(--color-iron-dark);

            .panel.negative & {
                color: var(--color-coral-darker);

                @media (prefers-color-scheme: dark) {
                    color: var(--color-coral-lighter);
                }
            }

            .panel.gooder & {
                color: var(--color-emerald-darker);

                @media (prefers-color-scheme: dark) {
                    color: var(--color-emerald-lighter);
                }
            }
        }

        @media (max-width: 768px) {
            text-align: center;
            position: relative;

            // gives bottom line after the <p> tag (description)
            :last-child::after {
                content: "";
                display: block;
                inline-size: 100%;
                height: 1px;
                background: var(--color-iron-lighter);
                margin-block-start: calc(var(--spacing) * 1);
            }
        }
    }

    .content {
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * var(--panel-content-spacing, 4));
    }
</style>
