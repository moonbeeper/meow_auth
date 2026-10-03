<script lang="ts">
    // "Complex binding patterns require an initialization value" when lang="ts" is not set... OH CMON!
    import { Accordion, Collapsible } from "bits-ui";
    import type { Snippet } from "svelte";
    import type { HTMLButtonAttributes } from "svelte/elements";
    import { fly, scale, slide } from "svelte/transition";

    type CollapsibleProps = HTMLButtonAttributes & Props & {};

    type Props = {
        children: Snippet;
        collapsible?: boolean;
        collapsible_bg?: boolean;
        inner_content?: Snippet;
    };

    let {
        children,
        collapsible = false,
        collapsible_bg = true,
        inner_content,
        ...rest
    }: CollapsibleProps | Props = $props();
</script>

{#if collapsible}
    <Accordion.Item>
        <Accordion.Trigger>
            {#snippet child({ props })}
                <button class="panel" {...props} {...rest}>
                    {@render children()}
                </button>
            {/snippet}
        </Accordion.Trigger>

        {#if inner_content}
            <Accordion.Content forceMount class="panel--collapsible">
                {#snippet child({ props, open })}
                    {#if open}
                        <div transition:slide={{ duration: 100 }} {...props}>
                            <div class={["collapsible", { collapsible_bg }]}>
                                {@render inner_content()}
                            </div>
                        </div>
                    {/if}
                {/snippet}
            </Accordion.Content>
        {/if}
    </Accordion.Item>
{:else}
    <div class="panel">
        {@render children()}
    </div>
{/if}

<style lang="scss">
    .panel {
        // all: unset; // kills everything. great.
        // --border-stuffies: 1px solid var(--color-iron-dark);
        display: flex;
        inline-size: 100%;
        padding-block: calc(var(--spacing) * 3);
        padding-inline: calc(var(--spacing) * 4);
        align-items: center;
        gap: calc(var(--spacing) * 3);
        text-align: start;
        // cursor: pointer;
        border: none;
        // border-inline: var(--border-stuffies);
        background: transparent;
        transition: background-color 0.1s ease-out;
        font: inherit;
        color: inherit;

        // &:first-child {
        //     border-top-left-radius: var(--typical-radius);
        //     border-top-right-radius: var(--typical-radius);
        //     border-top: var(--border-stuffies);
        // }

        // &:last-child {
        //     border-bottom-left-radius: var(--typical-radius);
        //     border-bottom-right-radius: var(--typical-radius);
        //     border-bottom: var(--border-stuffies);
        // }

        &:hover {
            --logItem-hover-background: color-mix(
                in oklab,
                var(--color-accent-lightest) 50%,
                transparent 50%
            );
            background: var(--logItem-hover-background);

            @media (prefers-color-scheme: dark) {
                --logItem-hover-background: color-mix(
                    in oklab,
                    var(--color-accent-light) 50%,
                    transparent 50%
                );
            }
        }
    }

    .panel--collapsible {
        overflow: hidden;
        padding-block-start: calc(var(--spacing) * 0);
        padding-block-end: calc(var(--spacing) * 3);
        padding-inline-start: calc(var(--spacing) * 6);
        padding-inline-end: calc(var(--spacing) * 4);

        @media (max-width: 768px) {
            padding-inline-start: calc(var(--spacing) * 4);
        }
    }

    .collapsible_bg {
        --content-background: color-mix(in oklab, var(--color-iron-light) 20%, transparent);
    }

    .collapsible {
        display: flex;
        border: 1px solid var(--color-iron-light);
        background: var(--content-background, var(--color-body));
        border-radius: calc(var(--typical-radius) * 0.6);
        padding: calc(var(--spacing) * 2);
    }
</style>
