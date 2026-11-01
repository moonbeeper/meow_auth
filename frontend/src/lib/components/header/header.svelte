<script lang="ts" module>
    import { refreshAll } from "$app/navigation";
    // Could use https://data-slot.com/
    import { page } from "$app/state";
    import { fade, slide, type TransitionConfig } from "svelte/transition";

    import logo from "#lib/assets/logo.svg";
    import logo_dark from "#lib/assets/logo_dark.svg";
    import { auth } from "#lib/auth/auth.svelte.js";
    import { UserFlag } from "#lib/auth/userFlags.js";

    import Avatar from "../avatar.svelte";
    import Button from "../button.svelte";
    import Link from "./link.svelte";

    let hasSession = $derived.by(() => {
        return auth.user != null;
    });

    let isSuperAdmin = $derived.by(() => {
        return auth.user?.flags.has(UserFlag.SuperAdmin) ?? false;
    });
    let cannotManageApps = $derived.by(() => {
        return auth.user?.flags.has(UserFlag.CannotManageOauthApplications) ?? false;
    });
    // const hasSession = true;

    export type NavLink = {
        href: string[];
        label: string;
        includeChildren: boolean;
    };

    let links = $derived.by(() => {
        const links: NavLink[] = [
            { href: ["/me", "/me/audit-log"], label: "Account", includeChildren: false },
            { href: ["/me/security"], label: "Security", includeChildren: false }
        ];

        if (!cannotManageApps) {
            links.push({ href: ["/me/my-apps"], label: "My apps", includeChildren: true });
        }

        links.push({
            href: ["/me/authorized-apps"],
            label: "Authorized apps",
            includeChildren: false
        });
        if (isSuperAdmin) {
            links.push({ href: ["/me/admin"], label: "Admin", includeChildren: false }); // should only show whenn... THEY ARE AND
        }

        return links;
    });

    let isAdminPath = $derived.by(() => {
        return page.url.pathname.startsWith("/me/admin/");
    });

    let adminLinks = $derived.by(() => {
        const id = page.url.pathname.match(/^\/me\/admin\/user\/([^/]+)/)?.[1];
        if (!id) {
            return [];
        }

        const base = "/me/admin/user/" + id;
        const links: NavLink[] = [
            { href: [base], label: "Overview", includeChildren: false },
            { href: [base + "/audit-log"], label: "Audit log", includeChildren: false },
            { href: [base + "/oauth-apps"], label: "OAuth apps", includeChildren: true },
            { href: [base + "/authorizations"], label: "Authorized apps", includeChildren: true }
        ];

        return links;
    });
    function slideFade(node: Element): TransitionConfig {
        const s = slide(node, { axis: "y", duration: 200 });

        return {
            delay: s.delay,
            duration: s.duration,
            easing: s.easing,
            css: (t, u) => `
                  ${s.css?.(t, u) ?? ""};
                  opacity: ${t};
              `
        };
    }
    $effect.root(() => {
        if (!hasSession) {
            refreshAll();
        }
    });
</script>

<header class="header">
    <div class="to-content">
        <Button href="#main">Skip to content</Button>
    </div>
    <nav class="nav" data-session={hasSession}>
        <a href="/" class="nav__logo">
            <picture>
                <source srcset={logo_dark} media="(prefers-color-scheme: dark)" />
                <img src={logo} alt="logo" />
            </picture>
        </a>

        {#if hasSession}
            <div class="nav__user">
                <!-- <div class="avatar2">?</div> -->
                <Avatar small interactive />
            </div>

            <div class="nav__links">
                {#each links as link}
                    <Link {link} />
                {/each}
            </div>

            {#if isAdminPath}
                <div class="nav__links below" transition:slideFade>
                    {#each adminLinks as link}
                        <Link {link} />
                    {/each}
                </div>
            {/if}
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

        &:focus-within {
            inset-inline-start: calc(var(--spacing) * 2);
        }
    }

    .nav {
        display: grid;
        grid-template-columns: auto 1fr;

        &[data-session="true"] {
            grid-template-areas: "logo user" "below below";
        }
    }

    .nav__logo {
        --focus-outline-offset: 4px; // better than using padding :]
        border-radius: 6px; // makes the focus ring rounded as the email logo
        margin-inline-start: var(--main-padding);
        user-select: none;

        .nav[data-session="true"] & {
            grid-area: logo;
        }
    }

    .nav__user {
        grid-area: user;
        justify-self: end;
        margin-block: auto;
        margin-inline-end: var(--main-padding);
    }

    .nav__links.below {
        grid-area: below;
        padding-inline: calc(var(--spacing) * 4);
        margin-inline: 0;
        margin-block-start: calc(var(--spacing) * 2);
        padding-block-end: calc(var(--spacing) * 1);
        border-bottom: 1px solid var(--color-iron-dark);
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

    @media (max-width: 768px) {
        .nav[data-session="true"] {
            grid-template-areas: "logo user" "links links" "below below";
        }
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
