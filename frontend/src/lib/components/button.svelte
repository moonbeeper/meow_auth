<script lang="ts">
    import type { ComponentProps, Snippet } from "svelte";
    import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";
    import { slide } from "svelte/transition";

    import Spinner from "./spinner.svelte";
    import Symbol from "./symbol.svelte";

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
        gooder?: boolean;
        disabled?: boolean;
        mute?: boolean;
        loading?: boolean;
        fontSize?: "normal" | "medium" | "small";
        children: Snippet;
        shouldFill?: boolean;
        square?: boolean;
        className?: string;
        symbol_prefix?: ComponentProps<typeof Symbol>;
        symbol_suffix?: ComponentProps<typeof Symbol>;
    };

    export type ButtonOrLinkProps = ButtonProps | LinkProps;

    // smh, typescript cant infer the type without doing as. man this union is uselss.
    let {
        children,
        primary = false,
        negative = false,
        gooder = false,
        mute = false,
        fontSize = "normal",
        disabled = false,
        loading = false,
        shouldFill = false,
        square = false,
        className,
        href,
        symbol_prefix,
        symbol_suffix,
        ...rest
    }: ButtonOrLinkProps = $props();

    let fontSizeClass = $derived.by(() => {
        return "font-" + fontSize;
    });
</script>

{#if href}
    <a
        class={[
            "button",
            fontSizeClass,
            { primary, negative, gooder, loading, shouldFill, square, mute },
            className
        ]}
        aria-disabled={disabled}
        {href}
        {...rest as HTMLAnchorAttributes}
    >
        {#if loading}
            <span class="spinner" transition:slide={{ axis: "x", duration: 200 }}>
                <Spinner />
            </span>
        {/if}

        {#if symbol_prefix && !loading}
            <span class="symbol" out:slide={{ axis: "x", duration: 200 }}>
                <Symbol {...symbol_prefix} />
            </span>
        {/if}

        {@render children()}

        {#if symbol_suffix && !loading}
            <span class="symbol" transition:slide={{ axis: "x", duration: 200 }}>
                <Symbol {...symbol_suffix} />
            </span>
        {/if}
    </a>
{:else}
    <button
        class={[
            "button",
            fontSizeClass,
            { primary, negative, gooder, loading, shouldFill, square, mute },
            className
        ]}
        aria-disabled={disabled}
        {disabled}
        {...rest as HTMLButtonAttributes}
    >
        {#if loading}
            <span class="spinner" transition:slide={{ axis: "x", duration: 200 }}>
                <Spinner />
            </span>
        {/if}

        {#if symbol_prefix && !loading}
            <span class="symbol" transition:slide={{ axis: "x", duration: 200 }}>
                <Symbol {...symbol_prefix} />
            </span>
        {/if}

        {@render children()}

        {#if symbol_suffix && !loading}
            <span class="symbol" transition:slide={{ axis: "x", duration: 200 }}>
                <Symbol {...symbol_suffix} />
            </span>
        {/if}
    </button>
{/if}

<style lang="scss">
    .button {
        --button-hover-brightness: 0.9;
        --button-font-size: var(--text-normal);
        border-radius: var(--typical-radius);
        border: var(--button-border-width, 1px) solid
            var(--button-border-color, var(--color-iron-medium));
        padding: var(--button-padding, calc(var(--spacing) * 2) 1rem);
        /* accent-color: var(--color-accent-medium); */
        color: var(--button-color, inherit);
        background: var(--button-background, var(--color-body));
        cursor: pointer;
        font-size: var(--button-font-size);
        font-weight: 600;
        display: inline-flex;
        transition-property:
            background-color, background, outline, border, opacity, color, transform;
        transition-duration: 0.3s, 0.3s, 0.1s, 0.1s, 0.1s, 0.1s, 0.1s;
        transition-timing-function: ease-out;
        justify-content: center;
        align-items: center;
        transform: scale(1);

        @media (prefers-color-scheme: dark) {
            --button-background: var(--color-body);
        }

        // makes so the line height does not change when the button is a link
        line-height: 1.1;
        gap: calc(var(--spacing) * 2);
        text-decoration: none;

        @media (any-hover: hover) {
            &:hover {
                filter: brightness(var(--button-hover-brightness));
            }
        }

        @media (prefers-color-scheme: dark) {
            --button-hover-brightness: 1.15;
        }

        &:active {
            transform: scale(0.99);
        }
    }

    .square {
        // --button-background: var(--color-iron-lightest);
        --button-background-muted: var(--color-iron-lighter);
        --button-color: inherit !important;
        --button-border-width: 0;
        flex: 1;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        padding: 1rem;
        border-radius: 0.4rem;
        text-align: center;
        white-space: nowrap;

        @media (prefers-color-scheme: dark) {
            // --button-background: var(--color-body);
            --button-background-muted: var(--color-body);
            --button-border-width: 1px;
        }

        // &.primary {
        //     --button-background: var(--color-accent-lighter);
        //     @media (prefers-color-scheme: dark) {
        //         --button-background: var(--color-accent-darker);
        //     }
        // }
    }

    .primary {
        --button-background: var(--color-accent-medium);
        --button-background-muted: var(--color-accent-lighter);
        --button-border-color: var(--color-body);
        --button-color: var(--color-iron-inverted);

        @media (prefers-color-scheme: dark) {
            --button-background: var(--color-accent-light);
            --button-background-muted: var(--color-accent-darker);
        }
    }

    .negative {
        --button-background: var(--color-coral-medium);
        --button-background-muted: var(--color-coral-lighter);
        --button-border-color: var(--color-body);
        --button-color: var(--color-iron-inverted);

        @media (prefers-color-scheme: dark) {
            --button-background: var(--color-coral-light);
            --button-background-muted: var(--color-coral-darker);
            --button-color-muted: var(--color-iron-darkest);
        }
    }

    .gooder {
        --button-background: var(--color-emerald-medium);
        --button-background-muted: var(--color-emerald-lighter);
        --button-border-color: var(--color-body);
        --button-color: var(--color-iron-inverted);

        @media (prefers-color-scheme: dark) {
            --button-background: var(--color-emerald-light);
            --button-background-muted: var(--color-emerald-darker);
        }
    }

    .font-medium {
        --button-font-size: var(--text-medium);
    }

    .font-small {
        --button-font-size: var(--text-small);
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

    .shouldFill {
        inline-size: 100%;
    }

    .button.square,
    .button.mute {
        --button-background: var(--button-background-muted);
        --button-color: var(--button-color-muted, var(--button-color, inherit));
    }

    .symbol {
        display: inline-flex;
        align-items: center;
        flex-shrink: 0;
    }
</style>
