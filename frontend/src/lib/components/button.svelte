<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";

    type ButtonProps = HTMLButtonAttributes &
        Props & {
            href?: null;
        };

    type LinkProps = HTMLAnchorAttributes &
        Props & {
            href: string;
        };

    type Props = {
        primary?: boolean;
        negative?: boolean;
        fontSize?: "normal" | "medium";
        children: Snippet;
    };

    // smh, typescript cant infer the type without doing as. man this union is uselss.
    let {
        children,
        primary = false,
        negative = false,
        fontSize = "normal",
        href,
        ...rest
    }: ButtonProps | LinkProps = $props();

    let fontSizeClass = $derived.by(() => {
        return "font-" + fontSize;
    });
</script>

{#if href}
    <a
        class={{ button: true, [fontSizeClass]: true }}
        class:primary
        class:negative
        {href}
        {...rest as HTMLAnchorAttributes}
    >
        {@render children()}
    </a>
{:else}
    <button
        class={{ button: true, [fontSizeClass]: true }}
        class:primary
        class:negative
        {...rest as HTMLButtonAttributes}
    >
        {@render children()}
    </button>
{/if}

<style lang="scss">
    .button {
        --button-hover-brightness: 0.9;
        --button-font-size: var(--text-normal);
        border-radius: var(--typical-radius);
        border: 1px solid var(--button-border-color, var(--color-iron-medium));
        padding: calc(var(--spacing) * 2) 1rem;
        /* accent-color: var(--color-accent-medium); */
        color: var(--button-color, inherit);
        background: var(--button-background, var(--color-body));
        cursor: pointer;
        font-size: var(--button-font-size);
        font-weight: 600;
        display: inline-flex;
        transition:
            background-color,
            outline,
            border,
            color 0.1s ease-out;
        justify-content: center;
        align-items: center;
        // makes so the line height does not change when the button is a link
        line-height: 1.1;

        @media (any-hover: hover) {
            &:hover {
                filter: brightness(var(--button-hover-brightness));
            }
        }

        @media (prefers-color-scheme: dark) {
            --button-hover-brightness: 1.1;
        }
    }

    .primary {
        --button-background: var(--color-accent-medium);
        @media (prefers-color-scheme: dark) {
            --button-background: var(--color-accent-light);
        }
        --button-border-color: var(--color-body);
        --button-color: var(--color-iron-inverted);
    }

    .negative {
        --button-background: var(--color-coral-medium);
        @media (prefers-color-scheme: dark) {
            --button-background: var(--color-coral-light);
        }
        --button-border-color: var(--color-body);
        --button-color: var(--color-iron-inverted);
    }

    .font-medium {
        --button-font-size: var(--text-medium);
    }
</style>
