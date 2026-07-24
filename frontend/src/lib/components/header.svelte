<script lang="ts">
    import logo from "$lib/assets/logo.svg";
    import logo_dark from "$lib/assets/logo_dark.svg";
    import { auth } from "$lib/auth.svelte";

    // let has_session = $derived.by(() => {
    //     return auth.user != null;
    // });
    const has_session = true;
</script>

<header class="header">
    <a href="#main" class="header--to-content button">Skip to content</a>
    <nav class="header--nav" data-session={has_session}>
        <a href="/" class="header--logo">
            <picture>
                <source srcset={logo_dark} media="(prefers-color-scheme: dark)" />
                <img src={logo} alt="logo" />
            </picture>
        </a>

        {#if has_session}
            <div class="header--user">
                <div class="avatar">?</div>
            </div>

            <div class="header--links">
                <a href="/me" class="header--link" aria-current="page">Account</a>
                <a href="/me/security" class="header--link">Security</a>
                <a href="/me/authorized-apps" class="header--link">Authorized apps</a>
                <a href="/me/my-apps" class="header--link">My apps</a>
                <a href="/me/admin" class="header--link">Admin</a>
            </div>
        {/if}
    </nav>
</header>
