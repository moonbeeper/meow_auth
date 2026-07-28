<script lang="ts">
    // Could use https://data-slot.com/
    import { page } from "$app/state";
    import logo from "$lib/assets/logo.svg";
    import logo_dark from "$lib/assets/logo_dark.svg";
    import { auth } from "$lib/auth.svelte";

    // let has_session = $derived.by(() => {
    //     return auth.user != null;
    // });
    const has_session = true;

    const links = [
        { href: "/me", label: "Account" },
        { href: "/me/security", label: "Security" },
        { href: "/me/authorized-apps", label: "Authorized apps" },
        { href: "/me/my-apps", label: "My apps" },
        { href: "/me/admin", label: "Admin" } // should only show whenn... THEY ARE AND
    ];
</script>

<header class="header">
    <a href="#main" class="to-content button">Skip to content</a>
    <nav class="nav" data-session={has_session}>
        <a href="/" class="nav__logo">
            <picture>
                <source srcset={logo_dark} media="(prefers-color-scheme: dark)" />
                <img src={logo} alt="logo" />
            </picture>
        </a>

        {#if has_session}
            <div class="nav__user">
                <div class="avatar">?</div>
            </div>

            <div class="nav__links">
                {#each links as link}
                    <a
                        href={link.href}
                        class="nav__link"
                        aria-current={page.url.pathname == link.href ? "page" : "false"}
                        >{link.label}</a
                    >
                {/each}
            </div>
        {/if}
    </nav>
</header>

<style lang="scss">
    .header {
        position: relative;
        z-index: 10;
        padding-block: calc(var(--spacing) * 3);
    }

    .to-content {
        position: absolute;
        inset-block-start: 4rem;
        inset-inline-start: -99999px;

        &:focus-visible {
            inset-inline-start: calc(var(--spacing) * 2);
        }
    }

    .nav {
        display: grid;
        grid-template-columns: auto 1fr;

        &[data-session="true"] {
            grid-template-areas: "logo user" "links links";
        }
    }

    .nav__logo {
        --focus-outline-offset: 4px; // better than using padding :]
        border-radius: 6px; // makes the focus ring rounded as the email logo
        margin-inline-start: var(--main-padding);

        .header--nav[data-session="true"] & {
            grid-area: logo;
        }
    }

    .nav__user {
        grid-area: user;
        justify-self: end;
        margin-block: auto;
        margin-inline-end: var(--main-padding);
    }

    .nav__links {
        grid-area: user;
        display: flex;
        gap: calc(var(--spacing) * 4);
        overflow-x: auto;
        scrollbar-width: none;
        margin-inline-start: calc(var(--spacing) * 6);
        align-items: center;
    }

    .nav__link {
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

    @media (max-width: 768px) {
        .nav[data-session="false"] {
            grid-template-columns: auto;
            justify-content: center;
        }

        .nav__logo {
            .nav[data-session="false"] & {
                margin-inline-start: 0;
            }
        }

        .nav__links {
            grid-area: links;
            padding-inline: calc(var(--spacing) * 4);
            margin-inline: 0;
            margin-block-start: calc(var(--spacing) * 2);
        }

        .to-content {
            &:focus-visible {
                inset-inline-start: calc(var(--spacing) * 6);
            }
        }
    }
</style>
