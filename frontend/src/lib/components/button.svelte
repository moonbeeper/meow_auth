<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";
    import { slide } from "svelte/transition";

    import Spinner from "./spinner.svelte";

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
        disabled?: boolean;
        loading?: boolean;
        fontSize?: "normal" | "medium";
        children: Snippet;
    };

    // smh, typescript cant infer the type without doing as. man this union is uselss.
    let {
        children,
        primary = false,
        negative = false,
        fontSize = "normal",
        disabled = false,
        loading = false,
        href,
        ...rest
    }: ButtonProps | LinkProps = $props();

    let fontSizeClass = $derived.by(() => {
        return "font-" + fontSize;
    });
</script>

{#if href}
    <a
        class={["button", fontSizeClass, { primary, negative, loading }]}
        aria-disabled={disabled}
        {href}
        {...rest as HTMLAnchorAttributes}
    >
        {@render children()}
    </a>
{:else}
    <button
        class={["button", fontSizeClass, { primary, negative, loading }]}
        aria-disabled={disabled}
        {disabled}
        {...rest as HTMLButtonAttributes}
    >
        {#if loading}
            <span class="spinner" transition:slide={{ axis: "x", duration: 200 }}>
                <Spinner />
            </span>
        {/if}
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
        transition-property: background-color, outline, border, opacity, color;
        transition-duration: 0.1s;
        transition-timing-function: ease-out;
        justify-content: center;
        align-items: center;
        // makes so the line height does not change when the button is a link
        line-height: 1.1;
        gap: calc(var(--spacing) * 2);

        @media (any-hover: hover) {
            &:hover {
                filter: brightness(var(--button-hover-brightness));
            }
        }

        @media (prefers-color-scheme: dark) {
            --button-hover-brightness: 1.1;
        }

        // &[disabled] {
        //     cursor: not-allowed;
        //     opacity: 0.5;
        // }
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

    .loading {
        cursor: progress !important;
    }

    .spinner {
        display: inline-flex;
        position: relative;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        inline-size: var(--button-font-size, 1em);
        block-size: var(--button-font-size, 1em);
        pointer-events: none;
    }
</style>
