<script lang="ts">
    import { page } from "$app/state";

    import type { NavLink } from "./header.svelte";

    type Props = {
        link: NavLink;
    };

    let { link }: Props = $props();

    let isCurrent = $derived.by(() => {
        let current = false;
        if (link.href.includes(page.url.pathname)) {
            current = true;
        }

        if (link.includeChildren) {
            for (let i = 0; i < link.href.length; i++) {
                if (page.url.pathname.startsWith(link.href[i] ?? "")) {
                    current = true;
                }
            }
        }
        return current;
    });
</script>

<a href={link.href[0]} class="link" aria-current={isCurrent ? "page" : "false"}>{link.label}</a>

<style lang="scss">
    .link {
        --hover-brightness: 0.9;
        --a-color: var(--link-color, var(--color-iron-darkest)) !important;
        position: relative;
        padding-block: calc(var(--spacing) * 1);
        font-size: var(--text-normal);
        font-weight: 500;
        white-space: nowrap;
        text-decoration: none;

        /* hover brightness from buttons */
        @media (any-hover: hover) {
            &:hover {
                filter: brightness(var(--hover-brightness));
            }
        }

        @media (prefers-color-scheme: dark) {
            --hover-brightness: 1.1;
        }

        &[aria-current="page"] {
            --link-color: var(--color-accent-dark);
            font-weight: 600;

            @media (prefers-color-scheme: dark) {
                --link-color: var(--color-accent-light);
            }

            &::after {
                position: absolute;
                content: "";
                inset-inline: 0;
                bottom: 0;
                height: 2px;
                background: var(--link-color);
            }
        }
    }
</style>
