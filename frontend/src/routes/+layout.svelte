<script lang="ts">
    import { version } from "$app/env";
    import favicon from "$lib/assets/favicon.svg";
    import logo from "$lib/assets/logo.svg";
    import logo_dark from "$lib/assets/logo_dark.svg";

    import "../styles/_global.scss";
    import "@fontsource-variable/inter/wght.css";

    let { children } = $props();
    let build_date = new Date(Number(version)).toTimeString();
</script>

<svelte:head>
    <link rel="icon" href={favicon} />
</svelte:head>
<header class="header">
    <a href="#main" class="header--to-content button">Skip to content</a>
    <nav class="header--nav">
        <a href="/" class="header--logo">
            <picture>
                <source srcset={logo_dark} media="(prefers-color-scheme: dark)" />
                <img src={logo} alt="logo" />
            </picture>
        </a>
    </nav>
</header>
{@render children()}
<footer class="footer">
    <div class="footer--content">Running on {__COMMIT_HASH__} built at {build_date}</div>
</footer>

<style lang="scss">
</style>
