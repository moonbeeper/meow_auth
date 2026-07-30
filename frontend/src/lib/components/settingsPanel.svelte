<script lang="ts">
    // TODO: i dont like the red colors ive chosen :(
    import type { Snippet } from "svelte";

    let {
        children,
        title = "Section Title",
        description = "i love space birbs. and hate space cats. i love space birbs (once again).",
        negative = false
    }: {
        children: Snippet;
        title: string;
        description: string;
        negative?: boolean;
    } = $props();
</script>

<section class={["panel", { negative }]}>
    <header class="header">
        <h2>{title}</h2>
        <p>{description}</p>
    </header>
    <div class="content">
        {@render children()}
    </div>
</section>

<style lang="scss">
    .panel {
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
        gap: calc(var(--spacing) * 4);
    }
</style>
